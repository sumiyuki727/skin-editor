import unittest
import tempfile
from pathlib import Path
from review_inventory import rustdoc_item, mask_rust, rust_lexical, python_symbols, merge_items, native_lexical, write_text


class ReviewInventoryTests(unittest.TestCase):
    def test_generated_inventory_uses_lf_even_on_windows(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'catalog.md';write_text(path,'one\ntwo\n')
            self.assertEqual(path.read_bytes(),b'one\ntwo\n')

    def test_redirect_is_not_a_duplicate_definition(self):
        self.assertIsNone(rustdoc_item('<html><script>location.replace("struct.X.html")</script></html>',
                                     'skin_editor_domain',Path('struct.X.html')))

    def fixture(self):
        return '''<main><a class="src rightside" href="../src/skin_editor_application/lib.rs.html#10-18">Source</a>
        <pre class="rust item-decl"><code>pub struct Session { value: <a title="struct skin_editor_domain::SkinProject">SkinProject</a> }</code></pre>
        <section id="impl-Clone-for-Session"><a class="src" href="../src/skin_editor_application/lib.rs.html#9">Source</a><h3 class="code-header">impl Clone for Session</h3></section>
        <section id="method.submit"><a class="src" href="../src/skin_editor_application/lib.rs.html#21-29">Source</a><h4 class="code-header">pub fn submit(&amp;mut self) -&gt; bool</h4></section>
        <section id="method.type_id"><a class="src" href="https://doc.rust-lang.org/src/core/any.rs.html#2-5">Source</a><h4 class="code-header">fn type_id()</h4></section></main>'''

    def test_own_methods_fields_and_impls_have_compiler_source(self):
        item=rustdoc_item(self.fixture(),'skin_editor_application',Path('struct.Session.html'))
        self.assertEqual(item['source'],dict(path='crates/application/src/lib.rs',line=10,end=18))
        self.assertEqual(item['references'],['skin_editor_domain::SkinProject'])
        self.assertEqual(item['methods'][0]['signature'],'pub fn submit(&mut self) -> bool')
        self.assertEqual([r['name'] for r in item['methods']],['submit'])
        self.assertEqual(item['impls'][0]['signature'],'impl Clone for Session')

    def test_required_trait_method_and_nested_module_are_not_lost(self):
        text=self.fixture().replace('pub struct Session','pub trait Session').replace('method.submit','tymethod.submit')
        item=rustdoc_item(text,'skin_editor_application',Path('ports/trait.Session.html'))
        self.assertEqual(item['path'],'skin_editor_application::ports::Session')
        self.assertEqual(item['methods'][0]['source']['line'],21)

    def test_expand_button_text_does_not_corrupt_declared_fields(self):
        text=self.fixture().replace('value: ', '<details><summary class="hideme">Show 14 fields</summary>value: ').replace(' }</code>',' }</details></code>')
        item=rustdoc_item(text,'skin_editor_application',Path('struct.Session.html'))
        self.assertNotIn('Show 14',item['declaration'])
        self.assertIn('value: SkinProject',item['declaration'])

    def test_inline_script_and_external_method_do_not_create_fake_items(self):
        self.assertIsNone(rustdoc_item('<main><pre class="item-decl">struct External</pre></main>',
                                     'skin_editor_domain',Path('struct.External.html')))

    def test_mask_preserves_lines_and_drops_nested_comments_and_raw_literals(self):
        text='/* fn wrong() { /* struct Bad */ } */\nconst X: &str = r###"fn fake() {}"###;\nfn real() {}'
        mask=mask_rust(text)
        self.assertEqual(mask.count('\n'),text.count('\n'))
        self.assertNotIn('wrong',mask);self.assertNotIn('fake',mask)
        self.assertEqual([(r['kind'],r['name'],r['line']) for r in rust_lexical(text)],
                         [('const','X',2),('fn','real',3)])

    def test_const_function_and_lifetime_are_not_misread_as_string_or_constant(self):
        rows=rust_lexical("pub const fn get<'a>(x: &'a str) -> char { '\\n' }\nfn next() {}")
        self.assertEqual([(r['kind'],r['name']) for r in rows],[('fn','get'),('fn','next')])

    def test_test_only_trait_impl_has_a_lexical_location_without_claiming_resolution(self):
        rows=rust_lexical('#[cfg(test)]\nimpl Port for Fake { fn request(&self) {} }')
        self.assertEqual(rows[0],dict(kind='impl',name='impl Port for Fake',line=2))
        self.assertEqual(rows[1]['name'],'request')

    def test_python_class_bases_methods_nested_functions_and_imports(self):
        text='from a import b\nclass Child(Base):\n def f(self, x: int=1):\n  def inner():\n   return b()\n  return self.g(x)\n'
        result=python_symbols(text)
        self.assertEqual(result['imports'],['a'])
        self.assertEqual(result['symbols'][0]['bases'],['Base'])
        self.assertEqual([r['name'] for r in result['symbols']],['Child','Child.f','Child.f.inner'])
        self.assertIn('self.g',result['symbols'][1]['calls'])
        self.assertNotIn('b',result['symbols'][1]['calls'])

    def test_reexport_alias_does_not_duplicate_its_defining_object(self):
        item=rustdoc_item(self.fixture(),'skin_editor_application',Path('struct.Session.html'))
        alias=dict(item,path='skin_editor_ui::values::Session',crate='skin_editor_ui')
        result=merge_items([alias,item])
        self.assertEqual(len(result),1)
        self.assertEqual(result[0]['crate'],'skin_editor_application')
        self.assertEqual(result[0]['aliases'],['skin_editor_ui::values::Session'])
        self.assertEqual(len(result[0]['methods']),1)

    def test_reexport_collapsed_span_and_resolved_self_are_one_definition(self):
        item=rustdoc_item(self.fixture(),'skin_editor_application',Path('struct.Session.html'))
        item['methods'][0]['signature']='pub fn submit(&mut self) -> Self'
        alias=dict(item,path='skin_editor_ui::Session',crate='skin_editor_ui',
                   source=dict(item['source'],end=10),
                   methods=[dict(item['methods'][0],signature='pub fn submit(&mut self) -> Session',
                                 references=['skin_editor_ui::Session'])])
        result=merge_items([alias,item])
        self.assertEqual(len(result),1)
        self.assertEqual(result[0]['source']['end'],18)
        self.assertEqual(len(result[0]['methods']),1)
        self.assertEqual(result[0]['methods'][0]['signature'],'pub fn submit(&mut self) -> Self')
        self.assertEqual(result[0]['methods'][0]['references'],['skin_editor_ui::Session'])

    def test_methods_on_same_derive_line_keep_distinct_trait_implementations(self):
        text='''<main><a class="src" href="../src/skin_editor_application/lib.rs.html#10">Source</a>
        <pre class="item-decl">struct Value;</pre>'''
        for trait in ['Debug','Display']:
            text+='''<details class="implementors-toggle"><summary><section id="impl-'''+trait+'''-for-Value">
            <h3 class="code-header">impl '''+trait+''' for Value</h3></section></summary>
            <section id="method.fmt"><a class="src" href="../src/skin_editor_application/lib.rs.html#9">Source</a>
            <h4 class="code-header">fn fmt(&amp;self)</h4></section></details>'''
        item=rustdoc_item(text+'</main>','skin_editor_application',Path('struct.Value.html'))
        self.assertEqual(len(item['methods']),2)
        self.assertEqual([r['implementation'] for r in item['methods']],
                         ['impl Debug for Value','impl Display for Value'])

    def test_native_collector_and_shader_functions_are_only_lexical_entries(self):
        text='static int observe(int pid) { return pid; }\nstruct Lease { int id; };\n'
        self.assertEqual([(r['kind'],r['name']) for r in native_lexical(text,'.c')],
                         [('function','observe'),('struct','Lease')])
        self.assertEqual([r['name'] for r in native_lexical('fn draw() {}\nstruct Pixel {}','.wgsl')],['draw','Pixel'])


if __name__=='__main__':unittest.main()
