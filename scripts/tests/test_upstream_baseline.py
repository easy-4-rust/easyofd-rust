"""Fixed-source generation rejects unsafe or incomplete reference inputs."""
import os,pathlib,subprocess,tempfile,unittest
ROOT=pathlib.Path(__file__).resolve().parents[2]
class Baseline(unittest.TestCase):
    def test_wrong_sha_and_missing_font_fail_without_source_changes(self):
        with tempfile.TemporaryDirectory() as d:
            p=pathlib.Path(d)
            subprocess.run(['git','init','-q',d],check=True)
            subprocess.run(['git','-C',d,'-c','user.name=test','-c','user.email=test@example.invalid','commit','--allow-empty','-qm','baseline'],check=True)
            sha=subprocess.check_output(['git','-C',d,'rev-parse','HEAD'],text=True).strip()
            env={**os.environ,'OFDRW_SOURCE_DIR':d}
            script=ROOT/'scripts/generate_upstream_text_fixtures.sh'
            self.assertNotEqual(subprocess.run(['bash',str(script),str(p/'out')],env=env,capture_output=True).returncode,0)
            env['OFDRW_COMMIT']=sha
            self.assertNotEqual(subprocess.run(['bash',str(script),str(p/'out')],env=env,capture_output=True).returncode,0)
            self.assertFalse((p/'out').exists())
            self.assertEqual(subprocess.check_output(['git','-C',d,'status','--porcelain'],text=True),'')
    def test_reverse_rejects_garbage_pfx(self):
        import shutil
        source=os.environ.get('OFDRW_SOURCE_DIR')
        if not source:self.skipTest('needs explicit fixed Java source')
        with tempfile.TemporaryDirectory() as d:
            for p in (ROOT/'target/bc185/rust').iterdir():
                if p.is_file():shutil.copyfile(p,pathlib.Path(d)/p.name)
            (pathlib.Path(d)/'public-test.p12').write_bytes(b'not a PFX')
            result=subprocess.run(['bash',str(ROOT/'scripts/verify_bc185_interop.sh'),d],capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0,'nonempty garbage PFX was accepted')
if __name__=='__main__':unittest.main()
