"""Refresh the translator template from authored UI source strings (not game data)."""
import ast,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def labels():
    values=set()
    for path in (ROOT/'mod/src').glob('*.rs'):
        if path.name in ('language.rs','localization.rs'):continue
        for token in re.findall(r'"(?:[^"\\]|\\.)*"',path.read_text()):
            try:value=ast.literal_eval(token)
            except (ValueError,SyntaxError):continue
            if len(value)>1 and any(c.isalpha() for c in value) and '\x00' not in value and '\t' not in value:
                # Formatted strings are translated as whole static fragments;
                # actual numeric values are preserved by the runtime renderer.
                for fragment in re.split(r'\{[^}]*\}',value):
                    fragment=fragment.strip()
                    if fragment and any(c.isalpha() for c in fragment):values.add(fragment.replace('\n','\\n'))
    return sorted(values)
if __name__=='__main__':
    target=ROOT/'assets/locales/eng.tsv'
    target.write_text('# MHGU Utility UI translations. English phrase<TAB>translated phrase.\n# Missing keys fall back to English. Keep controller prefixes unchanged.\n'+''.join(f'{s}\t{s}\n' for s in labels()))
