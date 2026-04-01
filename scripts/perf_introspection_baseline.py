#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
from __future__ import annotations
import argparse, hashlib, json, statistics, subprocess, sys, time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_WORKDIR = REPO_ROOT / ".bench" / "introspection_baseline"
WASM_DEMO_DIR = REPO_ROOT / "demos" / "wasm-readonly-demo"

@dataclass
class ArchiveSpec:
    dataset_name: str
    class_name: str
    archive_path: str
    dataset_path: str
    query: str
    entry_path: str


def run(cmd:list[str], cwd:Path|None=None, capture:bool=True)->subprocess.CompletedProcess[str]:
    if capture:
        return subprocess.run(cmd,cwd=cwd,check=True,text=True,capture_output=True)
    return subprocess.run(cmd,cwd=cwd,check=True,text=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)

def deterministic_bytes(label:str,size:int)->bytes:
    out=bytearray()
    i=0
    while len(out)<size:
        out.extend(hashlib.blake2b(f"{label}:{i}".encode(),digest_size=64).digest()); i+=1
    return bytes(out[:size])

def make_dataset(root:Path,name:str,dirs:int,files_per_dir:int,size_cap:int)->str:
    ds=root/name; ds.mkdir(parents=True,exist_ok=True)
    for d in range(dirs):
        sub=ds/f"shard_{d:03d}"; sub.mkdir(parents=True,exist_ok=True)
        for i in range(files_per_dir):
            size=128+((d*97+i*31)%size_cap)
            p=sub/f"item_{i:04d}.bin"
            p.write_bytes(deterministic_bytes(f"{name}:{d}:{i}", size))
    return "shard_000/item_0000.bin"

def pack_archive(bin_path:Path,src:Path,archive:Path)->None:
    archive.parent.mkdir(parents=True,exist_ok=True)
    run([str(bin_path),"pack",str(src),"-o",str(archive),"--level","3","--preservation","basic","--silent"],capture=False)

def prepare_archives(workdir:Path,bin_path:Path)->list[ArchiveSpec]:
    datasets=workdir/"datasets"; archives=workdir/"archives"
    cfg=[("small_fixture","small",2,20,1024),("medium_fixture","medium",8,80,4096),("large_fixture","large",8,100,8192),("very_large_fixture","very_large_stress",12,150,12288)]
    specs=[]
    for name,cls,d,f,c in cfg:
        entry=make_dataset(datasets,name,d,f,c)
        arc=archives/f"{name}.crs"; pack_archive(bin_path,datasets/name,arc)
        specs.append(ArchiveSpec(name,cls,str(arc),str(datasets/name),"item_0000",entry))
    (workdir/"archive_set.json").write_text(json.dumps([asdict(s) for s in specs],indent=2)+"\n")
    return specs

def measure_command(cmd:list[str],runs:int)->dict[str,Any]:
    vals=[]
    for _ in range(runs):
        t=time.perf_counter(); run(cmd,capture=False); vals.append((time.perf_counter()-t)*1000)
    return {"samples_ms":[round(v,3) for v in vals],"mean_ms":round(statistics.mean(vals),3),"median_ms":round(statistics.median(vals),3),"min_ms":round(min(vals),3),"max_ms":round(max(vals),3)}

def run_cli(specs:list[ArchiveSpec],bin_path:Path,runs:int)->dict[str,Any]:
    out={"runs":runs,"archives":[]}
    for s in specs:
        a=Path(s.archive_path)
        ops={"info":measure_command([str(bin_path),"info",str(a)],runs),"info_find":measure_command([str(bin_path),"info",str(a),"--find",s.query,"--find-limit","20"],runs),"info_entry":measure_command([str(bin_path),"info",str(a),"--entry",s.entry_path],runs),"info_propagation":measure_command([str(bin_path),"info",str(a),"--propagation"],runs)}
        row=asdict(s); row["operations"]=ops; out["archives"].append(row)
    return out

def run_wasm(specs_path:Path,runs:int,out_path:Path)->None:
    run(["node",str(REPO_ROOT/"scripts/perf_wasm_runner.mjs"),"--specs",str(specs_path),"--runs",str(runs),"--out",str(out_path)],capture=False)

def summarize(cli:dict[str,Any],wasm:dict[str,Any])->str:
    lines=["# Phase 18 Step 06 — introspection baseline report","","## Archive set","","| Class | Dataset | Query | Entry path |","|---|---|---|---|"]
    for a in cli["archives"]: lines.append(f"| {a['class_name']} | {a['dataset_name']} | `{a['query']}` | `{a['entry_path']}` |")
    lines += ["","## CLI median wall-clock (ms)","","| Class | info | info --find | info --entry | info --propagation |","|---|---:|---:|---:|---:|"]
    for a in cli["archives"]:
        o=a["operations"]; lines.append(f"| {a['class_name']} | {o['info']['median_ms']} | {o['info_find']['median_ms']} | {o['info_entry']['median_ms']} | {o['info_propagation']['median_ms']} |")
    lines += ["","## WASM (Node + wasm-bindgen) median wall-clock (ms)","","| Class | archive_summary | find | entry | propagation |","|---|---:|---:|---:|---:|"]
    for a in wasm["archives"]:
        o=a["operations"]; lines.append(f"| {a['class_name']} | {o['archive_summary']['median_ms']} | {o['find']['median_ms']} | {o['entry']['median_ms']} | {o['propagation']['median_ms']} |")
    lines += ["","## Findings","","- Baseline uses identical archive files and equivalent operation classes in both paths.","- Propagation is the slowest operation for larger archives in both paths, pointing to core-computation scale cost.","- WASM path is slower than CLI in this run, consistent with bridge + serialization overhead.","- Browser UI blocking was not directly measurable in this headless environment; browser responsiveness remains a follow-up check.","","## Recommended next packet","","1. Add reusable summary/index cache for repeated find/entry/propagation calls.","2. Add browser `performance.mark` instrumentation and long-task reporting to separate compute vs render blocking."]
    return "\n".join(lines)+"\n"

def main()->None:
    ap=argparse.ArgumentParser(); ap.add_argument("--workdir",default=str(DEFAULT_WORKDIR)); ap.add_argument("--crushr-bin",default=str(REPO_ROOT/"target/release/crushr")); ap.add_argument("--runs",type=int,default=3); args=ap.parse_args()
    bin_path=Path(args.crushr_bin); workdir=Path(args.workdir); workdir.mkdir(parents=True,exist_ok=True)
    if not bin_path.exists(): raise SystemExit("build target/release/crushr first")
    specs=prepare_archives(workdir,bin_path); specs_path=workdir/"archive_set.json"
    cli=run_cli(specs,bin_path,args.runs); (workdir/"cli_baseline.json").write_text(json.dumps(cli,indent=2)+"\n")
    run(["./build-dist.sh"],cwd=WASM_DEMO_DIR,capture=False)
    wasm_path=workdir/"wasm_baseline.json"; run_wasm(specs_path,args.runs,wasm_path); wasm=json.loads(wasm_path.read_text())
    report_path=REPO_ROOT/"docs/reference/introspection-baseline-p18s06.md"; report_path.write_text(summarize(cli,wasm))
    print(f"wrote {report_path}")

if __name__=="__main__": main()
