"""Offline architecture-review inventory; compiler rustdoc + explicitly lexical fallback.

This is documentation tooling, not a Rust resolver or a product control API.
Generate rustdoc with private items first. No network, GUI or native devices.
"""
import argparse
import ast
from collections import Counter, defaultdict
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = {
    'skin_editor_domain': 'crates/domain',
    'skin_editor_application': 'crates/application',
    'skin_editor_audio': 'desktop/audio',
    'skin_editor_rendering': 'desktop/rendering',
    'skin_editor_preview': 'desktop/preview',
    'skin_editor_infrastructure': 'desktop/infrastructure',
    'skin_editor_ui': 'desktop/ui',
    'skin_editor_desktop': 'desktop/host',
}
VOID = {'area','base','br','col','embed','hr','img','input','link','meta','param','source','track','wbr'}


class Node:
    def __init__(self, tag='', attrs=(), parent=None):
        self.tag, self.attrs, self.parent, self.children = tag, dict(attrs), parent, []
    def walk(self):
        yield self
        for child in self.children:
            if isinstance(child, Node):
                yield from child.walk()
    def text(self):
        if self.tag=='button' or (self.tag=='summary' and self.has_class('hideme')):return ''
        return ''.join(c.text() if isinstance(c, Node) else c for c in self.children)
    def has_class(self, name):
        return name in self.attrs.get('class','').split()


class Document(HTMLParser):
    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.root=Node();self.current=self.root;self.feed(text)
    def handle_starttag(self, tag, attrs):
        node=Node(tag,attrs,self.current);self.current.children.append(node)
        if tag not in VOID:self.current=node
    def handle_startendtag(self, tag, attrs):
        self.current.children.append(Node(tag,attrs,self.current))
    def handle_endtag(self, tag):
        node=self.current
        while node.parent is not None:
            if node.tag==tag:
                self.current=node.parent;return
            node=node.parent
    def handle_data(self, data):self.current.children.append(data)


def clean(text):return ' '.join(text.split())


def source_link(node):
    for item in node.walk():
        if item.tag=='a' and item.has_class('src'):
            match=re.search(r'(?:^|/)src/(skin_editor_\w+)/(.+?)\.html#(\d+)(?:-(\d+))?$',item.attrs.get('href',''))
            if match and match[1] in PACKAGES:
                return dict(path=PACKAGES[match[1]]+'/src/'+unquote(match[2]),line=int(match[3]),end=int(match[4] or match[3]))
    return None


def header(node):
    return next((n for n in node.walk() if n.has_class('code-header')),None)


def references(node):
    if node is None:return []
    refs=set()
    for item in node.walk():
        title=item.attrs.get('title','')
        match=re.match(r'(?:struct|enum|trait|type|fn|union) (skin_editor_\w+::.+)',title)
        if match:refs.add(match[1])
    return sorted(refs)


def implementation_context(node):
    parent=node.parent
    while parent is not None:
        if parent.has_class('implementors-toggle'):
            summary=next((c for c in parent.children if isinstance(c,Node) and c.tag=='summary'),None)
            sig=header(summary) if summary is not None else None
            return clean(sig.text()) if sig is not None else ''
        parent=parent.parent
    return ''


def unique_members(rows):
    # Reexport pages can resolve Self and collapse spans to a single line.
    # Preserve the defining page's signature; distinguish separate trait impls
    # even when derive puts same-named methods on one source line.
    result={}
    for row in rows:
        key=(row['name'],row['source']['path'],row['source']['line'],row.get('implementation',''))
        if key not in result:result[key]=dict(row)
        else:result[key]['references']=sorted(set(result[key]['references'])|set(row['references']))
    return list(result.values())


def rustdoc_item(text, crate, relative):
    match=re.match(r'(struct|enum|trait|fn|type|constant|static|macro|union)\.(.+)\.html$',relative.name)
    if not match:return None
    tree=Document(text).root
    main=next((n for n in tree.walk() if n.tag=='main'),None)
    if main is None:return None  # Compiler redirect pages are not extra definitions.
    decl=next((n for n in main.walk() if n.has_class('item-decl')),None)
    if decl is None:return None
    source=source_link(main)
    if source is None:return None
    path='::'.join((crate,*relative.parts[:-1],match[2]))
    methods=[];impls=[]
    for section in main.walk():
        identity=section.attrs.get('id','')
        if section.tag!='section':continue
        if identity.startswith(('method.','tymethod.')):
            sig=header(section);location=source_link(section)
            if sig is not None and location is not None:
                context=implementation_context(section)
                methods.append(dict(name=re.sub(r'\.\d+$','',identity.split('.',1)[1]),signature=clean(sig.text()),source=location,
                                    references=references(sig),implementation=context,trait_method=' for ' in context))
        elif identity.startswith('impl-'):
            sig=header(section);location=source_link(section)
            if sig is not None and location is not None:
                impls.append(dict(signature=clean(sig.text()),source=location,references=references(sig)))
    # A page can repeat an implementation reached via an alias. Source/signature
    # is the stable review key; no member is inferred from its spelling alone.
    unique=lambda rows:list({(r['signature'],r['source']['path'],r['source']['line']):r for r in rows}.values())
    return dict(path=path,crate=crate,kind=match[1],name=match[2],source=source,
                declaration=decl.text().strip(),references=references(decl),methods=unique_members(methods),impls=unique(impls))


def python_symbols(text):
    tree=ast.parse(text);symbols=[];imports=[]
    class Visitor(ast.NodeVisitor):
        def __init__(self):self.scope=[]
        def visit_ClassDef(self,node):
            symbols.append(dict(kind='class',name='.'.join(self.scope+[node.name]),line=node.lineno,end=node.end_lineno,
                                bases=[ast.unparse(b) for b in node.bases]))
            self.scope.append(node.name);self.generic_visit(node);self.scope.pop()
        def visit_FunctionDef(self,node):
            calls=[]
            class Calls(ast.NodeVisitor):
                def visit_Call(self,child):
                    calls.append(ast.unparse(child.func));self.generic_visit(child)
                def visit_FunctionDef(self,child):pass
                visit_AsyncFunctionDef=visit_FunctionDef
                def visit_ClassDef(self,child):pass
            # Nested definitions have their own entry; their calls are not
            # attributed to the enclosing function. Dispatch remains unresolved.
            collector=Calls()
            for statement in node.body:collector.visit(statement)
            symbols.append(dict(kind='function',name='.'.join(self.scope+[node.name]),line=node.lineno,end=node.end_lineno,
                                signature=('async ' if isinstance(node,ast.AsyncFunctionDef) else '')+'def '+node.name+'('+ast.unparse(node.args)+')'+(' -> '+ast.unparse(node.returns) if node.returns else ''),
                                calls=sorted(set(calls))))
            self.scope.append(node.name);self.generic_visit(node);self.scope.pop()
        visit_AsyncFunctionDef=visit_FunctionDef
        def visit_Import(self,node):imports.extend(a.name for a in node.names)
        def visit_ImportFrom(self,node):imports.append('.'*node.level+(node.module or ''))
    Visitor().visit(tree)
    return dict(symbols=symbols,imports=sorted(set(imports)))


def mask_rust(text):
    """Mask comments/strings while preserving offsets; not a type resolver."""
    result=list(text);i=0;n=len(text)
    def blank(a,b):
        for k in range(a,b):
            if result[k]!='\n':result[k]=' '
    while i<n:
        if text.startswith('//',i):
            j=text.find('\n',i);j=n if j<0 else j;blank(i,j);i=j
        elif text.startswith('/*',i):
            j=i+2;depth=1
            while j<n and depth:
                if text.startswith('/*',j):depth+=1;j+=2
                elif text.startswith('*/',j):depth-=1;j+=2
                else:j+=1
            blank(i,j);i=j
        elif (m:=re.match(r'(?:br|r)(#+)?"',text[i:])):
            end='"'+(m[1] or '');j=text.find(end,i+len(m[0]));j=n if j<0 else j+len(end);blank(i,j);i=j
        elif text[i]=='"' or (text[i]=="'" and re.match(r"'(?:\\[^\n]|[^'\\\n])'",text[i:])):
            quote=text[i];j=i+1
            while j<n:
                if text[j]=='\\':j+=2
                elif text[j]==quote:j+=1;break
                else:j+=1
            blank(i,min(j,n));i=j
        else:i+=1
    return ''.join(result)


def rust_lexical(text):
    masked=mask_rust(text);items=[]
    pattern=r'\b(struct|enum|trait|fn|type|const(?!\s+fn\b)|static|mod|macro_rules!)\s+([A-Za-z_]\w*)'
    for m in re.finditer(pattern,masked):
        items.append(dict(kind=m[1],name=m[2],line=masked.count('\n',0,m.start())+1,_offset=m.start()))
    for m in re.finditer(r'\bimpl\b',masked):
        end=masked.find('{',m.end())
        if end>=0:items.append(dict(kind='impl',name=clean(masked[m.start():end]),line=masked.count('\n',0,m.start())+1,_offset=m.start()))
    items.sort(key=lambda r:r['_offset'])
    for item in items:item.pop('_offset')
    return items


def merge_items(items):
    groups=defaultdict(list)
    for item in items:
        s=item['source'];groups[(item['kind'],item['name'],s['path'],s['line'])].append(item)
    merged=[]
    for rows in groups.values():
        owner=next((crate for crate,path in PACKAGES.items() if rows[0]['source']['path'].startswith(path+'/')),None)
        rows.sort(key=lambda r:(r['crate']!=owner,r['path'].count('::'),len(r['path']),r['path']))
        result=dict(rows[0]);result['aliases']=sorted({r['path'] for r in rows[1:]})
        result['methods']=unique_members([r for item in rows for r in item['methods']])
        result['impls']=list({(r['signature'],r['source']['path'],r['source']['line']):r for item in rows for r in item['impls']}.values())
        result['references']=sorted({r for item in rows for r in item['references']})
        merged.append(result)
    return merged


def native_lexical(text, suffix):
    masked=mask_rust(text);result=[]
    if suffix=='.wgsl':
        patterns=[('function',r'\bfn\s+(\w+)'),('struct',r'\bstruct\s+(\w+)')]
    else:
        patterns=[('function',r'(?m)^\s*(?:[A-Za-z_]\w*[\s*]+)+([A-Za-z_]\w*)\s*\([^;{}]*\)\s*\{'),
                  ('struct',r'\bstruct\s+(\w+)\s*\{')]
    for kind,pattern in patterns:
        for m in re.finditer(pattern,masked):
            if m[1] not in {'if','for','while','switch'}:
                result.append(dict(kind=kind,name=m[1],line=masked.count('\n',0,m.start())+1))
    return sorted(result,key=lambda r:r['line'])


def link(source, label='源码'):
    # Catalogs live at docs/review/catalog; links remain repository-relative.
    return '['+label+':'+str(source['line'])+'](../../../'+source['path']+'#L'+str(source['line'])+')'


def cell(text):return clean(text).replace('|','\\|').replace('`','&#96;')


def write_text(path, text):
    with Path(path).open('w',encoding='utf8',newline='\n') as stream:stream.write(text)


def load_metadata(path):
    data=json.loads(Path(path).read_text(encoding='utf8'))
    return [dict(name=p['name'],path=Path(p['manifest_path']).resolve().parent.relative_to(ROOT).as_posix(),
                 dependencies=[dict(name=d['name'],optional=d['optional'],features=d['features']) for d in p['dependencies']])
            for p in data['packages'] if p.get('source') is None]


def generate(output, metadata):
    output=Path(output);output.mkdir(parents=True,exist_ok=True)
    items=[]
    for crate,directory in PACKAGES.items():
        doc=ROOT/('target/doc' if directory.startswith('crates/') else 'desktop/target/doc')/crate
        if not doc.is_dir():raise ValueError('Generate private-item rustdoc first: '+str(doc))
        for page in sorted(doc.rglob('*.html')):
            if re.match(r'(struct|enum|trait|fn|type|constant|static|macro|union)\.',page.name):
                item=rustdoc_item(page.read_text(encoding='utf8'),crate,page.relative_to(doc))
                if item:items.append(item)
    items=merge_items(items)
    names={name:i['path'] for i in items for name in [i['path'],*i['aliases']]}
    for item in items:
        for row in [item,*item['methods'],*item['impls']]:
            row['references']=sorted({names.get(ref,ref) for ref in row['references']})
    inbound=defaultdict(set)
    for item in items:
        for ref in item['references']:
            if ref in names:inbound[names[ref]].add(item['path']+' (声明)')
        for method in item['methods']:
            for ref in method['references']:
                if ref in names:inbound[names[ref]].add(item['path']+'::'+method['name'])
    for item in items:item['signature_referenced_by']=sorted(inbound[item['path']])
    files=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard'],cwd=ROOT,text=True,encoding='utf8').splitlines()
    source_files=[]
    for name in sorted(set(files)):
        path=ROOT/name
        if not path.is_file() or path.suffix not in {'.rs','.py','.c','.wgsl'}:continue
        text=path.read_text(encoding='utf8');row=dict(path=name,sha256=hashlib.sha256(path.read_bytes()).hexdigest(),lines=len(text.splitlines()))
        if path.suffix=='.py':row.update(python_symbols(text))
        elif path.suffix=='.rs':
            row['symbols']=rust_lexical(text)
            row['imports']=[clean(m[0]) for m in re.finditer(r'\buse\b[^;]*;',mask_rust(text))]
        else:row['symbols']=native_lexical(text,path.suffix)
        source_files.append(row)
    packages={p['name']:p for path in metadata for p in load_metadata(path)}
    result=dict(schema=1,date='2026-10-30',method='Rust: compiler rustdoc(private,all features), explicit own source only; Python: AST; other source/test fallback: lexical, not name resolution',
                cfg_test_not_in_rustdoc=True,rustdoc_does_not_resolve_call_bodies=True,
                packages=list(packages.values()),rust_items=items,source_files=source_files)
    write_text(output/'CODE_INDEX.json',json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    by_crate=defaultdict(list)
    for item in items:by_crate[item['crate']].append(item)
    for crate,rows in by_crate.items():
        out=['# '+crate+'：编译器符号清单','',
             '快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。',
             '引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。','']
        for item in sorted(rows,key=lambda x:x['path']):
            out+=['## '+item['path'],'',link(item['source'])+' · '+item['kind'],'','```rust',item['declaration'],'```','']
            if item['aliases']:out+=['同一定义的重导出／别名（不重复计作对象）：'+', '.join('`'+r+'`' for r in item['aliases']), '']
            if item['references']:out+=['声明引用：'+', '.join('`'+r+'`' for r in item['references']), '']
            if item['signature_referenced_by']:
                out+=['反向签名引用（仅已解析的项目类型链接，非完整调用图）：'+', '.join('`'+r+'`' for r in item['signature_referenced_by']), '']
            if item['impls']:
                out+=['实现／约束（显式或 derive，无类继承）：','']
                out += ['- `'+cell(r['signature'])+'` — '+link(r['source']) for r in item['impls']]
                out+=['']
            if item['methods']:
                out+=['| 方法签名 | 所属实现 | 来源 | 签名类型引用 |','| --- | --- | --- | --- |']
                out += ['| `'+cell(r['signature'])+'` | '+cell(r['implementation'])+' | '+link(r['source'])+' | '+cell(', '.join(r['references']))+' |' for r in item['methods']]
                out+=['']
        write_text(output/(crate+'.md'),'\n'.join(out)+'\n')
    # The exact file-level index covers tests, auxiliary tools and historical P0
    # code, rather than pretending rustdoc compiled cfg(test) or inactive branches.
    out=['# 全部源码与辅助代码索引','',
         'Rust 表是屏蔽注释／字符串后的词法定义位置；不保证泛型、宏展开或 cfg 的语义解析。Python 表来自 AST，基类和调用拼写并非动态派发证明。C／WGSL 仅记录有限词法声明，需结合原文件审阅。',
         'P0、测试及诊断不因此成为产品模块；本表不执行代码或扫描用户目录。','']
    for row in source_files:
        out+=['## '+row['path'],'',str(row['lines'])+' 行；SHA256 `'+row['sha256']+'`','']
        if 'imports' in row:out+=['导入：'+', '.join('`'+r+'`' for r in row['imports']), '']
        out+=['| 定义 | 位置／签名 |','| --- | --- |']
        for symbol in row['symbols']:
            extra=symbol.get('signature') or ', '.join(symbol.get('bases',[]))
            out+=['| '+cell(symbol['kind']+' '+symbol['name'])+' | '+link(dict(path=row['path'],line=symbol['line']))+(' · `'+cell(extra)+'`' if extra else '')+' |']
        out+=['']
    write_text(output/'SOURCE_FILES.md','\n'.join(out)+'\n')
    stats=dict(rust_items=len(items),rust_methods=sum(len(x['methods']) for x in items),source_files=len(source_files),source_lines=sum(r['lines'] for r in source_files),by_crate=dict(Counter(i['crate'] for i in items)))
    write_text(output/'SUMMARY.json',json.dumps(stats,indent=2)+'\n')
    print(json.dumps(stats))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',default='docs/review/catalog')
    parser.add_argument('--metadata',action='append',required=True)
    args=parser.parse_args();generate(ROOT/args.output,args.metadata)
