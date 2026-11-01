/* Opt-in Windows diagnostic, never linked into the product. No ETL file.
 * Only scheduler transitions of our suspended child and its descendants survive
 * the native callback. Other event payloads/names/stacks are never retained.
 */
#define WIN32_LEAN_AND_MEAN
#define _CRT_SECURE_NO_WARNINGS
#include <windows.h>
#include <evntrace.h>
#include <evntcons.h>
#include <tdh.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <wchar.h>
#include <io.h>
#include <fcntl.h>
#include <limits.h>
#pragma comment(lib, "advapi32.lib")
#pragma comment(lib, "tdh.lib")

#define MAX_PROCESSES 16
#define MAX_THREADS 512
#define MAX_EVENTS 1000000
typedef struct { DWORD pid; BOOL live; } OwnedProcess;
typedef struct { DWORD pid, tid; BOOL ended, retired; } OwnedThread;
typedef struct { LONGLONG qpc; DWORD cpu, pid, tid, in, lifetime; } Switch;
static OwnedProcess processes[MAX_PROCESSES];
static OwnedThread threads[MAX_THREADS];
static Switch *events;
static ULONG process_count, thread_count, event_count, invalid, total_buffers;
/* Named error counts diagnose rejected captures; none relax validity. */
static ULONG property_errors, schema_errors, identity_errors, capacity_errors;
static DWORD property_failures[8][6];
static ULONG property_failure_count;
static DWORD identity_failures[8][5];
static ULONG identity_failure_count;
static ULONGLONG callback_qpc, callback_count;
static BOOL checked_version[256];
static TRACEHANDLE consumer;
static DWORD observed_runner;
static const GUID thread_guid = {0x3d6fa8d1,0xfe05,0x11d0,{0x9d,0xda,0,0xc0,0x4f,0xd7,0xba,0x7c}};
static const GUID process_guid = {0x3d6fa8d0,0xfe05,0x11d0,{0x9d,0xda,0,0xc0,0x4f,0xd7,0xba,0x7c}};

static int own_process(DWORD pid) {
    ULONG i;
    for (i=0;i<process_count;i++) if (processes[i].pid==pid && processes[i].live) return (int)i;
    return -1;
}
static int own_thread(DWORD tid) {
    ULONG i;
    for (i=thread_count;i>0;i--) if (threads[i-1].tid==tid && !threads[i-1].retired) return (int)(i-1);
    return -1;
}
static void thread_event(DWORD pid,DWORD tid,UCHAR op) {
    int i=own_thread(tid);
    if(op==4) return; /* DCEnd is rundown, not exit. */
    if(op==2) { if(i>=0 && threads[i].pid==pid) threads[i].ended=TRUE; return; }
    if(i>=0) {
        if(threads[i].pid==pid && !threads[i].ended) return; /* Start/DCStart duplicate. */
        if(threads[i].ended && (threads[i].pid!=pid || op==1)) {
            /* An actual Start after End gets a new capture-local lifetime,
               even for the same PID/TID. Rundown alone cannot resurrect it. */
            threads[i].retired=TRUE;
        } else {
            /* Concurrent identity change or rundown resurrection is invalid. */
            if(identity_failure_count<8) { DWORD *a=identity_failures[identity_failure_count++];a[0]=1;a[1]=threads[i].pid;a[2]=tid;a[3]=op;a[4]=threads[i].ended?1:2; }
            invalid++;identity_errors++;return;
        }
    }
    if(pid==observed_runner || own_process(pid)<0) return; /* Parent is admission only; never retain a foreign identity. */
    if(thread_count==MAX_THREADS) {invalid++;capacity_errors++;return;}
    threads[thread_count].pid=pid;threads[thread_count].tid=tid;thread_count++;
}
static BOOL property(EVENT_RECORD *r, const wchar_t *name, DWORD *value) {
    PROPERTY_DATA_DESCRIPTOR d = {0};
    ULONG size=0;
    d.PropertyName=(ULONGLONG)(ULONG_PTR)name; d.ArrayIndex=ULONG_MAX;
    ULONG code=TdhGetPropertySize(r,0,NULL,1,&d,&size);
    if(code==ERROR_SUCCESS && size==sizeof(*value)) code=TdhGetProperty(r,0,NULL,1,&d,sizeof(*value),(PBYTE)value);
    if(code!=ERROR_SUCCESS || size!=sizeof(*value)) {
        if(property_failure_count<8) {
            DWORD *a=property_failures[property_failure_count++];
            a[0]=r->EventHeader.ProviderId.Data1; a[1]=r->EventHeader.EventDescriptor.Opcode;
            a[2]=r->EventHeader.EventDescriptor.Version; a[3]=code; a[4]=size;
            a[5]=wcscmp(name,L"ProcessId")==0?0:wcscmp(name,L"TThreadId")==0?1:2;
        }
        invalid++; property_errors++; return FALSE;
    }
    return TRUE;
}
static BOOL switch_schema(EVENT_RECORD *r) {
    /* Verify the live OS schema before using the documented two-u32 prefix.
       No private offsets, guessed version fallback, or repeated TDH per switch. */
    BYTE storage[16384];
    TRACE_EVENT_INFO *info=(TRACE_EVENT_INFO *)storage;
    ULONG size=sizeof(storage), i;
    const wchar_t *names[2]={L"NewThreadId",L"OldThreadId"};
    if (TdhGetEventInformation(r,0,NULL,info,&size)!=ERROR_SUCCESS || info->TopLevelPropertyCount<2) return FALSE;
    for (i=0;i<2;i++) {
        EVENT_PROPERTY_INFO *p=&info->EventPropertyInfoArray[i];
        if (p->Flags || p->count!=1 || p->length!=4 || p->nonStructType.InType!=TDH_INTYPE_UINT32 ||
            p->NameOffset>=size || wcscmp((wchar_t *)(storage+p->NameOffset),names[i])) return FALSE;
    }
    return TRUE;
}
static void emit_switch(LONGLONG qpc, DWORD cpu, DWORD tid, DWORD in) {
    int i=own_thread(tid);
    Switch *e;
    if (i<0) return;
    if (event_count==MAX_EVENTS) { invalid++; capacity_errors++; return; }
    e=&events[event_count++];
    e->qpc=qpc; e->cpu=cpu; e->pid=threads[i].pid; e->tid=tid; e->in=in;
    e->lifetime=(DWORD)i+1;
}
static void collect_record(EVENT_RECORD *r) {
    UCHAR op=r->EventHeader.EventDescriptor.Opcode;
    DWORD pid,tid,parent;
    int i;
    if (IsEqualGUID(&r->EventHeader.ProviderId,&process_guid) && (op==1 || op==2)) {
        if (!property(r,L"ProcessId",&pid)) return;
        if(pid==GetCurrentProcessId()) return;
        if (op==2) {
            i=own_process(pid);
            if(i>=0) { ULONG j;processes[i].live=FALSE;for(j=0;j<thread_count;j++) if(threads[j].pid==pid) threads[j].ended=TRUE; }
            return;
        }
        if (!property(r,L"ParentId",&parent) || own_process(parent)<0) return;
        for(i=0;i<(int)process_count;i++) if(processes[i].pid==pid) break;
        if (i<(int)process_count) {
            if(identity_failure_count<8) { DWORD *a=identity_failures[identity_failure_count++];a[0]=0;a[1]=pid; }
            invalid++; identity_errors++; return;
        }
        if (process_count==MAX_PROCESSES) { invalid++; capacity_errors++; return; }
        processes[process_count].pid=pid; processes[process_count++].live=TRUE;
    } else if (IsEqualGUID(&r->EventHeader.ProviderId,&thread_guid)) {
        if (op==36) {
            DWORD pair[2];
            UCHAR version=r->EventHeader.EventDescriptor.Version;
            if (!checked_version[version]) {
                if (!switch_schema(r)) { invalid++; schema_errors++; return; }
                checked_version[version]=TRUE;
            }
            if (r->UserDataLength<sizeof(pair)) { invalid++; schema_errors++; return; }
            memcpy(pair,r->UserData,sizeof(pair));
            emit_switch(r->EventHeader.TimeStamp.QuadPart,r->BufferContext.ProcessorIndex,pair[1],0);
            emit_switch(r->EventHeader.TimeStamp.QuadPart,r->BufferContext.ProcessorIndex,pair[0],1);
        } else if (op==1 || op==2 || op==3 || op==4) {
            if (!property(r,L"ProcessId",&pid) || !property(r,L"TThreadId",&tid)) return;
            /* The ordinary runner is only a parent admission identity. Its
               already running threads have no complete capture boundary. */
            thread_event(pid,tid,op);
        }
    }
}
static void WINAPI record(EVENT_RECORD *r) {
    LARGE_INTEGER a,b;
    QueryPerformanceCounter(&a); collect_record(r); QueryPerformanceCounter(&b);
    callback_count++; callback_qpc+=(ULONGLONG)(b.QuadPart-a.QuadPart);
}
static ULONG WINAPI buffer(EVENT_TRACE_LOGFILEW *log) {
    (void)log; total_buffers++; return TRUE;
}
static DWORD WINAPI consume(void *unused) {
    (void)unused; return ProcessTrace(&consumer,1,NULL,NULL);
}
static DWORD enable_privilege(const wchar_t *name) {
    HANDLE token=NULL; TOKEN_PRIVILEGES p={0}; DWORD result;
    if (!OpenProcessToken(GetCurrentProcess(),TOKEN_ADJUST_PRIVILEGES|TOKEN_QUERY,&token)) return GetLastError();
    p.PrivilegeCount=1;
    if (!LookupPrivilegeValueW(NULL,name,&p.Privileges[0].Luid)) { result=GetLastError(); CloseHandle(token); return result; }
    p.Privileges[0].Attributes=SE_PRIVILEGE_ENABLED;
    SetLastError(ERROR_SUCCESS);
    if (!AdjustTokenPrivileges(token,FALSE,&p,0,NULL,NULL)) result=GetLastError(); else result=GetLastError();
    CloseHandle(token); return result; /* Process-local token; no policy/ACL changes. */
}
static BOOL token_summary(HANDLE process, DWORD *elevated, DWORD *kind) {
    HANDLE token=NULL; TOKEN_ELEVATION e={0}; TOKEN_ELEVATION_TYPE type=TokenElevationTypeDefault;
    DWORD size; BOOL ok;
    if(!OpenProcessToken(process,TOKEN_QUERY,&token)) return FALSE;
    ok=GetTokenInformation(token,TokenElevation,&e,sizeof(e),&size) &&
       GetTokenInformation(token,TokenElevationType,&type,sizeof(type),&size);
    if(ok) { *elevated=e.TokenIsElevated; *kind=(DWORD)type; }
    CloseHandle(token); return ok;
}
static HANDLE limited_token(void) {
    HANDLE token=NULL,primary=NULL; TOKEN_LINKED_TOKEN linked={0}; DWORD size;
    BYTE a[sizeof(TOKEN_USER)+SECURITY_MAX_SID_SIZE], b[sizeof(TOKEN_USER)+SECURITY_MAX_SID_SIZE];
    TOKEN_ELEVATION e={0}; TOKEN_ELEVATION_TYPE type=TokenElevationTypeDefault;
    BOOL ok=FALSE;
    if(!OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&token)) return NULL;
    if(GetTokenInformation(token,TokenLinkedToken,&linked,sizeof(linked),&size)) {
        ok=GetTokenInformation(token,TokenUser,a,sizeof(a),&size) &&
           GetTokenInformation(linked.LinkedToken,TokenUser,b,sizeof(b),&size) &&
           EqualSid(((TOKEN_USER *)a)->User.Sid,((TOKEN_USER *)b)->User.Sid) &&
           GetTokenInformation(linked.LinkedToken,TokenElevation,&e,sizeof(e),&size) && !e.TokenIsElevated &&
           GetTokenInformation(linked.LinkedToken,TokenElevationType,&type,sizeof(type),&size) && type==TokenElevationTypeLimited;
    }
    CloseHandle(token);
    if(!ok) { if(linked.LinkedToken) CloseHandle(linked.LinkedToken); SetLastError(ERROR_ACCESS_DENIED); return NULL; }
    /* Process creation requires a primary token, not an identification handle.
       Duplicate only the already verified same-user non-elevated token. */
    if(!DuplicateTokenEx(linked.LinkedToken,TOKEN_QUERY|TOKEN_DUPLICATE|TOKEN_ASSIGN_PRIMARY,NULL,
                         SecurityImpersonation,TokenPrimary,&primary)) {
        DWORD error=GetLastError();CloseHandle(linked.LinkedToken);SetLastError(error);return NULL;
    }
    CloseHandle(linked.LinkedToken);return primary;
}
static BOOL same_user(HANDLE process) {
    HANDLE a=NULL,b=NULL; DWORD size; BOOL ok=FALSE;
    BYTE x[sizeof(TOKEN_USER)+SECURITY_MAX_SID_SIZE],y[sizeof(TOKEN_USER)+SECURITY_MAX_SID_SIZE];
    if(OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&a) && OpenProcessToken(process,TOKEN_QUERY,&b))
        ok=GetTokenInformation(a,TokenUser,x,sizeof(x),&size) && GetTokenInformation(b,TokenUser,y,sizeof(y),&size) &&
           EqualSid(((TOKEN_USER *)x)->User.Sid,((TOKEN_USER *)y)->User.Sid);
    if(a) CloseHandle(a); if(b) CloseHandle(b); return ok;
}
static ULONGLONG created_at(HANDLE process) {
    FILETIME a,b,c,d;
    if(!GetProcessTimes(process,&a,&b,&c,&d)) return 0;
    return ((ULONGLONG)a.dwHighDateTime<<32)|a.dwLowDateTime;
}
static int lifetime_self_test(void) {
    BOOL ok=TRUE;int i;
    processes[0]=(OwnedProcess){11,TRUE};processes[1]=(OwnedProcess){22,TRUE};process_count=2;
    thread_event(11,101,1);thread_event(11,101,3);ok&=thread_count==1 && !invalid;
    thread_event(11,101,4);ok&=!threads[0].ended;
    thread_event(11,101,2);thread_event(99,101,1);ok&=own_thread(101)<0 && thread_count==1 && !invalid;
    thread_event(22,101,1);i=own_thread(101);ok&=i==1 && threads[i].pid==22 && !invalid;
    thread_event(99,101,1);ok&=invalid==1; /* live identity collision rejected */
    invalid=identity_errors=0;
    thread_event(22,101,2);thread_event(22,101,1);i=own_thread(101);
    ok&=!invalid && thread_count==3 && i==2 && threads[1].retired && threads[i].pid==22;
    thread_event(22,101,2);thread_event(22,101,3);ok&=invalid==1; /* rundown is not a new lifetime */
    invalid=identity_errors=0;observed_runner=11;
    thread_event(11,101,1);ok&=own_thread(101)<0 && thread_count==3 && !invalid;
    printf("{\"schema\":1,\"lifetime_self_test\":%s}\n",ok?"true":"false");return ok?0:1;
}
int wmain(int argc, wchar_t **argv) {
    BYTE storage[sizeof(EVENT_TRACE_PROPERTIES)+512]={0};
    EVENT_TRACE_PROPERTIES *p=(EVENT_TRACE_PROPERTIES *)storage;
    EVENT_TRACE_LOGFILEW log={0};
    TRACEHANDLE session=0;
    wchar_t name[128], *command=NULL;
    PROCESS_INFORMATION child={0}; STARTUPINFOW startup={sizeof(startup)};
    LARGE_INTEGER qpc,freq,start,finish;
    HANDLE worker=NULL,job=NULL,file=INVALID_HANDLE_VALUE,linked=NULL;
    HANDLE ready=NULL,done_event=NULL;
    ULONGLONG runner_created=0;
    LPWCH environment=NULL;
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limits={0};
    DWORD status=0, stop_status=0, trace_status=0, child_status=~0u, privilege, timeout;
    DWORD child_elevated=~0u, child_token_type=0;
    FILE *out=NULL; ULONG i; BOOL started=FALSE, resumed=FALSE, limited=FALSE, observe=FALSE;
    if(argc==2 && !wcscmp(argv[1],L"--self-test")) return lifetime_self_test();
    if(argc==8 && !wcscmp(argv[3],L"--observe")) observe=TRUE;
    else if (argc==6 && !wcscmp(argv[5],L"--limited")) limited=TRUE;
    else if(argc!=5) { fwprintf(stderr,L"usage: measure_cpu_trace output.json timeout_ms cwd child_command [--limited]\n"); return 2; }
    timeout=wcstoul(argv[2],NULL,10);
    if (timeout<1000 || timeout>(observe?240000u:120000u) || GetActiveProcessorCount(ALL_PROCESSOR_GROUPS)>64) return 2;
    file=CreateFileW(argv[1],GENERIC_WRITE,0,NULL,CREATE_NEW,FILE_ATTRIBUTE_NORMAL,NULL);
    if(file==INVALID_HANDLE_VALUE) return 2;
    out=_fdopen(_open_osfhandle((intptr_t)file,_O_WRONLY|_O_TEXT),"w");
    if(!out) { CloseHandle(file); return 2; }
    events=calloc(MAX_EVENTS,sizeof(Switch)); if(!observe) command=_wcsdup(argv[4]);
    QueryPerformanceFrequency(&freq); QueryPerformanceCounter(&qpc); start=qpc;
    swprintf(name,128,L"SkinEditorCpu-%lu-%lld",GetCurrentProcessId(),qpc.QuadPart);
    p->Wnode.BufferSize=sizeof(storage); p->Wnode.Flags=WNODE_FLAG_TRACED_GUID; p->Wnode.ClientContext=1;
    p->Wnode.Guid=(GUID){0x71fb706a,(WORD)GetCurrentProcessId(),0x4d10,{0x82,0xd5,0x69,0xd6,0x45,0x5e,0xa5,0xc1}};
    p->Wnode.Guid.Data1^=(DWORD)qpc.LowPart;
    p->LoggerNameOffset=sizeof(EVENT_TRACE_PROPERTIES); wcscpy((wchar_t *)(storage+p->LoggerNameOffset),name);
    p->BufferSize=64; p->MinimumBuffers=32; p->MaximumBuffers=128; p->FlushTimer=1;
    p->LogFileMode=EVENT_TRACE_REAL_TIME_MODE|EVENT_TRACE_SYSTEM_LOGGER_MODE;
    p->EnableFlags=EVENT_TRACE_FLAG_CSWITCH|EVENT_TRACE_FLAG_THREAD|EVENT_TRACE_FLAG_PROCESS;
    privilege=enable_privilege(SE_SYSTEM_PROFILE_NAME);
    if(!events || (!observe && !command)) { status=ERROR_OUTOFMEMORY; goto done; }
    if(observe) {
        wchar_t prefix[128]; DWORD exit_code;
        observed_runner=wcstoul(argv[4],NULL,10); runner_created=_wcstoui64(argv[5],NULL,10);
        swprintf(prefix,128,L"Local\\SkinEditorCpu-%lu-%llu-",observed_runner,runner_created);
        if(!observed_runner || !runner_created || wcslen(argv[6])>240 || wcslen(argv[7])>240 ||
           wcsncmp(argv[6],prefix,wcslen(prefix)) || wcsncmp(argv[7],prefix,wcslen(prefix)) || !wcscmp(argv[6],argv[7])) {
            status=ERROR_INVALID_PARAMETER;goto done;
        }
        child.dwProcessId=observed_runner;
        child.hProcess=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,FALSE,observed_runner);
        if(!child.hProcess || created_at(child.hProcess)!=runner_created || !same_user(child.hProcess) ||
           !GetExitCodeProcess(child.hProcess,&exit_code) || exit_code!=STILL_ACTIVE ||
           !token_summary(child.hProcess,&child_elevated,&child_token_type) || child_elevated ||
           (child_token_type!=TokenElevationTypeDefault && child_token_type!=TokenElevationTypeLimited)) {
            status=ERROR_ACCESS_DENIED;goto done;
        }
        ready=OpenEventW(EVENT_MODIFY_STATE,FALSE,argv[6]); done_event=OpenEventW(SYNCHRONIZE,FALSE,argv[7]);
        if(!ready || !done_event) {status=GetLastError();goto done;}
    }
    status=StartTraceW(&session,name,p); if(status) goto done; started=TRUE;
    if(observe) goto admitted;
    job=CreateJobObjectW(NULL,NULL); limits.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if(!job || !SetInformationJobObject(job,JobObjectExtendedLimitInformation,&limits,sizeof(limits))) {status=GetLastError();goto done;}
    startup.lpDesktop=L"winsta0\\default"; /* Same existing interactive desktop; never edit its ACL. */
    if(limited) {
        linked=limited_token(); if(!linked) {status=GetLastError();goto done;}
        status=enable_privilege(SE_INCREASE_QUOTA_NAME); if(status) goto done;
        /* Same-user linked token only. No credentials, policy/ACL changes or
           foreign shell tokens. Child logs to its own file, no copied std handles. */
        environment=GetEnvironmentStringsW(); if(!environment) {status=GetLastError();goto done;}
        if(!CreateProcessAsUserW(linked,NULL,command,NULL,NULL,FALSE,CREATE_SUSPENDED|CREATE_NO_WINDOW|CREATE_UNICODE_ENVIRONMENT,
                                environment,argv[3],&startup,&child)) {status=GetLastError();goto done;}
    } else {
        startup.dwFlags=STARTF_USESTDHANDLES;
        startup.hStdInput=GetStdHandle(STD_INPUT_HANDLE); startup.hStdOutput=GetStdHandle(STD_OUTPUT_HANDLE); startup.hStdError=GetStdHandle(STD_ERROR_HANDLE);
        if(!CreateProcessW(NULL,command,NULL,NULL,TRUE,CREATE_SUSPENDED|CREATE_NO_WINDOW,NULL,argv[3],&startup,&child)) {status=GetLastError();goto done;}
    }
    if(!token_summary(child.hProcess,&child_elevated,&child_token_type)) {status=GetLastError();goto done;}
    if(limited && (child_elevated || child_token_type!=TokenElevationTypeLimited)) {status=ERROR_ACCESS_DENIED;goto done;}
    if(!AssignProcessToJobObject(job,child.hProcess)) {status=GetLastError();goto done;}
admitted:
    processes[0].pid=child.dwProcessId; processes[0].live=TRUE; process_count=1;
    if(!observe) {threads[0].pid=child.dwProcessId; threads[0].tid=child.dwThreadId; thread_count=1;}
    log.LoggerName=name; log.ProcessTraceMode=PROCESS_TRACE_MODE_REAL_TIME|PROCESS_TRACE_MODE_EVENT_RECORD|PROCESS_TRACE_MODE_RAW_TIMESTAMP;
    log.EventRecordCallback=record; log.BufferCallback=buffer;
    consumer=OpenTraceW(&log); if(consumer==INVALID_PROCESSTRACE_HANDLE) {status=GetLastError();goto done;}
    worker=CreateThread(NULL,0,consume,NULL,0,NULL); if(!worker) {status=GetLastError();goto done;}
    if(observe) {
        HANDLE waits[2]={done_event,child.hProcess}; DWORD wait;
        if(!SetEvent(ready)) {status=GetLastError();goto done;}
        wait=WaitForMultipleObjects(2,waits,FALSE,timeout);
        if(wait!=WAIT_OBJECT_0) {status=wait==WAIT_TIMEOUT?WAIT_TIMEOUT:ERROR_PROCESS_ABORTED;goto done;}
        if(created_at(child.hProcess)!=runner_created) {status=ERROR_INVALID_DATA;goto done;}
        child_status=0;goto done;
    }
    if(ResumeThread(child.hThread)==~0u) {status=GetLastError();goto done;} resumed=TRUE;
    if(WaitForSingleObject(child.hProcess,timeout)!=WAIT_OBJECT_0) {status=WAIT_TIMEOUT;goto done;}
    if(!GetExitCodeProcess(child.hProcess,&child_status)) status=GetLastError();
done:
    if(environment) FreeEnvironmentStringsW(environment);
    if(linked) CloseHandle(linked);
    if(!observe && child.hProcess && (status || !resumed)) { TerminateProcess(child.hProcess,3); WaitForSingleObject(child.hProcess,5000); }
    /* Stop only this newly created session. Never attach/stop an existing logger. */
    if(started) stop_status=ControlTraceW(session,name,p,EVENT_TRACE_CONTROL_STOP);
    if(worker) {
        if(WaitForSingleObject(worker,10000)!=WAIT_OBJECT_0) { CloseTrace(consumer); WaitForSingleObject(worker,5000); status=WAIT_TIMEOUT; }
        if(!GetExitCodeThread(worker,&trace_status) || trace_status==STILL_ACTIVE) { fclose(out); return 3; }
        CloseHandle(worker);
    }
    if(consumer && consumer!=INVALID_PROCESSTRACE_HANDLE) CloseTrace(consumer);
    if(child.hThread) CloseHandle(child.hThread);
    if(child.hProcess) CloseHandle(child.hProcess);
    if(job) CloseHandle(job); /* Also destroys any still-running owned descendants. */
    if(ready) CloseHandle(ready); if(done_event) CloseHandle(done_event);
    QueryPerformanceCounter(&finish);
    fprintf(out,"{\"schema\":2,\"collector_pid\":%lu,\"root_pid\":%lu,\"status\":%lu,\"privilege_status\":%lu,\"stop_status\":%lu,\"trace_status\":%lu,\"child_status\":%lu,\"frequency\":%lld,\"span_qpc\":%lld,\"invalid\":%lu,\"events_lost\":%lu,\"buffers_lost\":%lu,\"buffer_count\":%lu,\"buffers_allocated\":%lu,\"thread_count\":%lu,\"events\":[",GetCurrentProcessId(),child.dwProcessId,status,privilege,stop_status,trace_status,child_status,freq.QuadPart,finish.QuadPart-start.QuadPart,invalid,p->EventsLost,p->RealTimeBuffersLost+p->LogBuffersLost,total_buffers,p->NumberOfBuffers,thread_count);
    for(i=0;i<event_count;i++) { Switch *e=&events[i]; fprintf(out,"%s[%lld,%lu,%lu,%lu,%lu,%lu]",i?",":"",e->qpc,e->cpu,e->pid,e->tid,e->in,e->lifetime); }
    fprintf(out,"],\"thread_lifetimes\":[");
    for(i=0;i<thread_count;i++) fprintf(out,"%s[%lu,%lu,%lu]",i?",":"",threads[i].pid,threads[i].tid,i+1);
    fprintf(out,"],\"callback_count\":%llu,\"callback_wall_qpc\":%llu,\"property_errors\":%lu,\"schema_errors\":%lu,\"identity_errors\":%lu,\"capacity_errors\":%lu,\"property_failures\":[",callback_count,callback_qpc,property_errors,schema_errors,identity_errors,capacity_errors);
    for(i=0;i<property_failure_count;i++) { DWORD *a=property_failures[i]; fprintf(out,"%s[%lu,%lu,%lu,%lu,%lu,%lu]",i?",":"",a[0],a[1],a[2],a[3],a[4],a[5]); }
    fprintf(out,"],\"identity_failures\":[");
    for(i=0;i<identity_failure_count;i++) { DWORD *a=identity_failures[i];fprintf(out,"%s[%lu,%lu,%lu,%lu,%lu]",i?",":"",a[0],a[1],a[2],a[3],a[4]); }
    fprintf(out,"],\"limited_requested\":%s,\"child_elevated\":%lu,\"child_token_type\":%lu,\"launch_api\":\"%s\",\"observed_runner\":%s,\"runner_created\":%llu}\n",limited?"true":"false",child_elevated,child_token_type,observe?"ordinary-runner-owned-children":limited?"CreateProcessAsUserW":"CreateProcessW",observe?"true":"false",runner_created); fclose(out); free(command); free(events);
    return status || stop_status || trace_status || child_status || invalid || p->EventsLost || p->RealTimeBuffersLost || p->LogBuffersLost ? 1 : 0;
}
