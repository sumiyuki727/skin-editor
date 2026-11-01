import copy,unittest,json,hashlib,tempfile
from pathlib import Path
from unittest.mock import patch
from measure_frame_dispatch import validate_spans,cost_regions,layout_passes,painter_partition,run_without_uac,NAMES
from measure_main_work import product_argument

def fixture():
    rows=[]
    for n in range(40):
        for kind in (1,2,3,4,5,11,13,14):
            start=100+n*100+kind*3;end=start+2
            rows.append([len(rows)+2,kind,start,end,start,end,4,4])
    return dict(schema=1,invalid=0,active=0,owners=0,spans=len(rows),rows=rows)

def painter_fixture():
    rows=[];passes=[]
    for n in range(40):
        a=100+n*100
        for k,x,y in [(15,a,a+90),*[(k,a+10*(i+1),a+10*(i+1)+2) for i,k in enumerate((1,2,3,4,16,18))]]:
            rows.append([len(rows)+2,k,x,y,x,y,4,4])
        passes.append(dict(row=[a,a,4,n,n+1,1,0],causes=[]))
    return dict(schema=2,invalid=0,active=0,owners=0,spans=len(rows),rows=rows,passes=passes,discard_sources=[])

class FrameDispatchTests(unittest.TestCase):
    def test_schema_2_adds_painter_scopes_without_reinterpreting_schema_1(self):
        d=painter_fixture();validate_spans(d);d['schema']=1
        with self.assertRaises(ValueError):validate_spans(d)
        self.assertEqual(validate_spans(fixture())[1],0)
    def test_painter_partition_checks_containment_before_exclusive_wall(self):
        rows=painter_fixture()['rows'];v=painter_partition(rows)
        self.assertEqual(v['painter_wall_ms']['count'],40)
        self.assertEqual(v['painter_wall_ms']['p50'],.00009)
        self.assertEqual(v['exclusive_wall_ms']['painter_other']['p50'],.000078)
        self.assertIn('not driver',v['scope'])
        for change in [lambda r:r[1].__setitem__(5,r[0][5]+1),lambda r:r[1].__setitem__(5,r[2][5]),
            lambda r:r.pop(5)]:
            values=copy.deepcopy(rows);change(values)
            with self.assertRaises(ValueError):painter_partition(values)
    def test_no_uac_mode_never_constructs_or_builds_collector_and_has_no_cpu_acceptance(self):
        with tempfile.TemporaryDirectory() as directory:
            out=Path(directory);binary=out/'fixture.exe';binary.write_bytes(b'fixture');sig=hashlib.sha256(binary.read_bytes()).hexdigest()
            data=painter_fixture();report=dict(ui_cost=[[0]]*4+[[40]])
            report.update({k:0 for k in ('missing_frames','pcm_peak_bytes','pcm_final_bytes','source_raw_peak','source_raw_final',
                'callback_resource_releases','endpoint_returns_in_render','endpoint_returns_outside_render','max_live_targets')})
            def observed(binary,path,*args,**kwargs):
                path.mkdir();(path/'framework_trace.json').write_text(json.dumps(data),encoding='utf8')
                return report,[dict(working_set_bytes=1,private_bytes=2)]
            with patch('measure_frame_dispatch.observe',side_effect=observed),patch('measure_frame_dispatch.fingerprint',return_value={}),\
                patch('measure_frame_dispatch.summarize',return_value={}),patch('measure_frame_dispatch.evaluate_gui',return_value=([],[],dict(zero_missing_frames=True))),\
                patch('measure_frame_dispatch.stream_checks',return_value={}),patch('measure_frame_dispatch.audit_checks',return_value={}),\
                patch('measure_frame_dispatch.Collector',side_effect=AssertionError('no elevation')),patch('measure_frame_dispatch.build_collector',side_effect=AssertionError('no collector')):
                run_without_uac(out,binary,{},sig)
            result=json.loads((out/'proof.json').read_text());self.assertFalse(result['uac_requested'])
            self.assertFalse(result['cpu_meter_validated']);self.assertIsNone(result['cpu_budget_passed']);self.assertFalse(result['p2_a3_accepted'])
    def test_old_flags_unchanged_and_new_modes_are_explicit(self):
        self.assertEqual(product_argument(False),'--verify-streamed-gui')
        self.assertEqual(product_argument(True),'--verify-main-work-gui')
        self.assertEqual(product_argument(False,'repaint'),'--verify-repaint-only-gui')
        self.assertEqual(product_argument(True,'dispatch'),'--verify-dispatch-gui')
        self.assertEqual(product_argument(False,'dispatch-repaint'),'--verify-dispatch-repaint-gui')
        with self.assertRaises(ValueError):product_argument(False,'dispatch')
        with self.assertRaises(ValueError):product_argument(True,'repaint')
    def test_actual_api_regions_match_only_owned_main_and_keep_uncertainty(self):
        data=fixture();rows,drift=validate_spans(data);self.assertEqual(drift,0)
        result=cost_regions(rows,1,2,1,5000,1_000_000_000,0,[(1,2,0,1,5000),(8,2,0,1,5000),(1,3,1,1,5000)],1)
        self.assertEqual(result['rendered_queue_submit']['count'],40)
        self.assertAlmostEqual(result['rendered_queue_submit']['cpu_one_core_percent'],8000/4999)
        self.assertGreater(result['rendered_present']['boundary_uncertainty_percent_upper'],0)
        self.assertIn('repaint_user_event',NAMES)
        self.assertNotIn('painter_total',result)
    def test_missing_cleanup_reversed_duplicate_unknown_and_clock_drift_fail(self):
        for f in [lambda d:d.update(owners=1),lambda d:d.update(active=1),lambda d:d.update(invalid=1),
            lambda d:d['rows'].pop(),lambda d:d['rows'][1].__setitem__(0,2),lambda d:d['rows'][0].__setitem__(1,15),
            lambda d:d['rows'][0].__setitem__(3,1),lambda d:d['rows'][0].__setitem__(5,999999),
            lambda d:d.update(spans=32769)]:
            d=fixture();f(d)
            with self.assertRaises(ValueError):validate_spans(d)
    def test_missing_normal_draw_or_overlapping_same_api_cannot_pass(self):
        rows=fixture()['rows'];rows=[r for r in rows if r[1]!=3]
        with self.assertRaises(ValueError):cost_regions(rows,1,2,1,5000,10**9,0,[],1)
        rows=fixture()['rows'];rows.append(copy.deepcopy(rows[0]))
        with self.assertRaises(ValueError):cost_regions(rows,1,2,1,5000,10**9,0,[],1)
    def test_partial_spans_are_clipped_and_parent_totals_are_not_subtracted(self):
        rows=fixture()['rows'];v=cost_regions(rows,1,2,105,5000,10**9,0,[(1,2,0,1,5000)],2)
        self.assertEqual(v['surface_acquire']['count'],39)
        self.assertEqual(v['encoder_finish']['count'],40)
        self.assertGreater(v['rendered_present']['cpu_one_core_percent'],0)
    def test_multiple_ui_passes_are_distinct_from_frames_and_sources_are_bounded(self):
        raw=dict(passes=[dict(row=[100,10,4,7,1,0,1],causes=[0]),dict(row=[101,11,4,7,2,1,0],causes=[])],
            discard_sources=[dict(file_utf8=list(b'layout/grid.rs'),line=12)])
        v=layout_passes(raw,2);self.assertEqual(v['whole_stage_frame_ids'],1);self.assertEqual(v['extra_ui_passes'],1)
        for change in [lambda d:d['passes'][1]['row'].__setitem__(4,1),lambda d:d['passes'][0]['causes'].__setitem__(0,1),
            lambda d:d['discard_sources'][0].__setitem__('file_utf8',list(b'C:/private')),lambda d:d['passes'].pop()]:
            d=copy.deepcopy(raw);change(d)
            with self.assertRaises(ValueError):layout_passes(d,2)

if __name__=='__main__':unittest.main()
