import copy
import unittest
from measure_gpu_path import path,ORDINALS

def markers():
    a=dict(schema=1,mode='markers',invalid=0,error=None,discarded=0,released=False,
        frames=[[i+1,*range(10+i*10,17+i*10)] for i in range(97)],samples=[],uploads=[],snapshots=[])
    b=copy.deepcopy(a);b['released']=True
    return a,b

def upload():
    a=dict(schema=1,mode='upload',invalid=0,error=None,discarded=0,released=False,frames=[],
        samples=[[100+n,o,5,2,12,18,1] for n,o in enumerate(ORDINALS)],
        uploads=[[n,r,1,2,3,100,9,105] for n in range(4) for r in range(4)],
        snapshots=[dict(stage=s,supported=False) for s in ('before_probe','created_probe')])
    b=copy.deepcopy(a);b['released']=True;b['snapshots'].append(dict(stage='released_probe',supported=False))
    return a,b

class GpuPathTests(unittest.TestCase):
    def test_actual_regions_are_named_without_submit_or_present_attribution(self):
        a,b=markers();v=path(a,b,'markers');self.assertEqual(v['count'],97)
        self.assertEqual(v['interval_ms']['surface_acquire_and_pass_setup']['p50'],.000001)
        self.assertIn('not CPU submit',v['scope']);self.assertNotIn('corrected_cpu',v)
    def test_missing_late_reversed_duplicate_and_overflow_edges_fail(self):
        for f in [lambda a:a['frames'][0].__setitem__(2,0),lambda a:a['frames'][1].__setitem__(0,1),
            lambda a:a['frames'][0].__setitem__(3,1),lambda a:a['frames'].pop(),lambda a:a.__setitem__('invalid',1)]:
            a,b=markers();f(a);b=copy.deepcopy(a);b['released']=True
            with self.assertRaises(ValueError):path(a,b,'markers')
        a,b=markers();b['frames'][0][7]+=1
        with self.assertRaises(ValueError):path(a,b,'markers')
    def test_upload_completion_can_precede_submit_return_but_not_begin(self):
        a,b=upload();v=path(a,b,'upload');self.assertEqual(v['timings_ms']['real_update_buffers']['count'],16)
        self.assertIn('extra workload',v['scope']);self.assertEqual(v['warm_calls_ms']['real_queue_submit']['count'],15)
        a['uploads'][0][7]=99;b['uploads'][0][7]=99
        with self.assertRaises(ValueError):path(a,b,'upload')
    def test_upload_plan_and_resource_release_cannot_be_silently_dropped(self):
        for f in [lambda a:a['uploads'].pop(),lambda a:a['uploads'][0].__setitem__(0,1),
            lambda a:a['samples'][0].__setitem__(4,32769),lambda a:a['samples'][1].__setitem__(1,34)]:
            a,b=upload();f(a);b.update(copy.deepcopy(a));b['released']=True
            with self.assertRaises(ValueError):path(a,b,'upload')
        a,b=upload();b['released']=False
        with self.assertRaises(ValueError):path(a,b,'upload')
        a,b=upload();b['snapshots'].pop()
        with self.assertRaises(ValueError):path(a,b,'upload')

if __name__=='__main__':unittest.main()
