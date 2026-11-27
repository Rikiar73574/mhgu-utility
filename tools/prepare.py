"""Prepare ignored build inputs from explicitly supplied local data and an owned dump."""
import argparse,hashlib,json,os,shutil,subprocess,sys,tempfile
from pathlib import Path
from paths import ROOT,DATA,GENERATED
from game_names import export,armor_names

def replace_artwork_cache(source_root):
    artwork=(('location-art','.img'),('weapon-art','.png'))
    # Stage both inputs before touching the two build-owned destinations. The
    # current build cache may itself be the explicitly selected input cache.
    with tempfile.TemporaryDirectory(dir=DATA,prefix='.artwork-') as temporary:
        staging=Path(temporary)
        for directory,suffix in artwork:
            destination=staging/directory;destination.mkdir()
            for source in (source_root/directory).glob('*'+suffix) if source_root else ():
                if source.is_file():shutil.copyfile(source,destination/source.name)
        for directory,_ in artwork:
            destination=DATA/directory
            if destination.is_symlink() or destination.is_file():destination.unlink()
            elif destination.exists():shutil.rmtree(destination)
            (staging/directory).replace(destination)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--catalog',type=Path)
    p.add_argument('--database',type=Path)
    p.add_argument('--game-dir',type=Path)
    p.add_argument('--art-dir',type=Path,help='optional local location-art/ and weapon-art/ cache; omitted artwork uses existing UI fallbacks')
    p.add_argument('--generated-dir',type=Path,help='reuse all ten pre-generated Rust data files unchanged')
    a=p.parse_args()
    DATA.mkdir(parents=True,exist_ok=True);GENERATED.mkdir(parents=True,exist_ok=True)
    if a.generated_dir:
        modules=('game_ui','quest_data','armor_data','catalog','world','weapon_data','item_data','map_pixels','ui_pixels','set_data')
        inputs=[a.generated_dir/f'{name}.rs' for name in modules]
        if not all(path.is_file() for path in inputs):p.error('generated-dir must contain all ten generated Rust modules')
        for source in inputs:shutil.copyfile(source,GENERATED/source.name)
        if a.catalog:
            shutil.copyfile(a.catalog,DATA/'catalog.json')
            locations=a.catalog.with_name('locations.json')
            if locations.is_file():shutil.copyfile(locations,DATA/'locations.json')
        manifest={'mode':'unchanged local generated inputs','generated_sha256':{path.name:hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}}
        (ROOT/'build/input-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        return
    if not all((a.catalog,a.database,a.game_dir)):
        p.error('provide --catalog, --database and --game-dir, or --generated-dir')
    if a.art_dir and not all((a.art_dir/name).is_dir() for name in ('location-art','weapon-art')):
        p.error('--art-dir must contain location-art/ and weapon-art/')
    catalog=json.loads(a.catalog.read_text())
    if catalog.get('schema_version')!=1 or not catalog.get('items') or not catalog.get('weapons'):
        p.error('expected the Kiranico catalog adapter schema_version 1')
    if not a.database.is_file():p.error('MHGenDatabase SQLite file is missing')
    table=a.game_dir/'romfs/nativeNX/eng/table'
    weapons,items=export(table)
    DATA.mkdir(parents=True,exist_ok=True);GENERATED.mkdir(parents=True,exist_ok=True)
    destination=DATA/'catalog.json'
    if a.catalog.resolve()!=destination.resolve():shutil.copyfile(a.catalog,destination)
    (DATA/'game-weapons.json').write_text(json.dumps(weapons,ensure_ascii=False))
    (DATA/'game-items.json').write_text(json.dumps(items,ensure_ascii=False))
    (DATA/'game-armor.json').write_text(json.dumps(armor_names(table/'armorSeriesData_eng.gmd'),ensure_ascii=False))
    replace_artwork_cache(a.art_dir)
    env=os.environ.copy();env['MHGU_DATABASE']=str(a.database.resolve());env['MHGU_GAME_DIR']=str(a.game_dir.resolve())
    def run(name,*args):subprocess.run([sys.executable,str(ROOT/'tools'/name),*map(str,args)],env=env,check=True)
    run('extract_map_pngs.py',a.game_dir,DATA/'location-maps')
    for script in ('world.py','embed.py','item_data.py','armor_data.py','quest_data.py','game_ui.py','map_art.py','ui_art.py','set_data.py'):run(script)
    manifest={'catalog_sha256':hashlib.sha256(a.catalog.read_bytes()).hexdigest(),
              'database_sha256':hashlib.sha256(a.database.read_bytes()).hexdigest(),
              'generated_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(GENERATED.glob('*.rs'))},
              'artwork':('Locally supplied illustrations/renders via --art-dir; no artwork is fetched.' if a.art_dir else
                         'Optional illustrations/renders omitted; bundled icons and extracted minimaps remain available.')}
    (ROOT/'build/input-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
if __name__=='__main__':main()
