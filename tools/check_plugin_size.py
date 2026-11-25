"""Keep the compendium inside the recomp's shared 12 MiB plugin region.
Reserve 3 MiB minus one 64 KiB loader page for peers and alignment.
The dedicated native Set Builder uses that extra page; this remains a
combined-loader constraint, not permission to assume every stack fits.
"""
import struct
from pathlib import Path
p=Path(__file__).resolve().parents[1]/'build/hunters_compendium.nro'
data=p.read_bytes();bss=struct.unpack_from('<I',data,0x38)[0]
footprint=(len(data)+bss+0xffff)&~0xffff
budget=9*1024*1024+0x10000
if footprint>budget:raise SystemExit(f'Compendium plugin footprint {footprint} exceeds {budget} byte budget')
print(f'Plugin footprint: {footprint} bytes / {budget} byte budget')
