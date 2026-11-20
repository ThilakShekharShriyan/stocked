"use client";

// The 3D agent world. One scene: the house agent in the middle, flatmates and their agents around
// it, specialised agents on the next ring, sponsor systems outside. Recorded events travel along
// the connections. Labels are real buttons, so the scene is keyboard- and screen-reader-usable,
// and the whole thing degrades to a list when WebGL is unavailable.

import { useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { TONE_HEX, Tone, WorldEdge, WorldNode } from "@/lib/world";

export type Pulse = { from: string; to: string; tone: Tone };

type Props = {
  nodes: WorldNode[];
  edges: WorldEdge[];
  pulses: Pulse[];
  selected: string | null;
  onSelect: (id: string | null) => void;
  /** Node ids to tint (waiting, refused, done). */
  tones?: Record<string, Tone>;
  /** Node ids that recently did something. */
  active?: string[];
  /** People shown as not taking part (faded). */
  absent?: string[];
  /** When set, everything else is dimmed. */
  emphasis?: string[];
  /** Node to move to when "Focus on current activity" is pressed. */
  currentId?: string | null;
  className?: string;
};

const KIND_COLOR: Record<WorldNode["kind"], string> = { hub: "#efe9dc", person: "#e7dcc3", agent: "#8fb08a", tool: "#86a8cb", checkpoint: "#d9a441", data: "#a39c8a" };

function place(n: WorldNode): THREE.Vector3 {
  const a = (n.angle * Math.PI) / 180;
  return new THREE.Vector3(Math.cos(a) * n.radius, n.height, Math.sin(a) * n.radius);
}

function arc(a: THREE.Vector3, b: THREE.Vector3): THREE.QuadraticBezierCurve3 {
  const mid = a.clone().add(b).multiplyScalar(0.5);
  mid.y += 0.9 + a.distanceTo(b) * 0.12;
  return new THREE.QuadraticBezierCurve3(a, mid, b);
}

function geometryFor(kind: WorldNode["kind"]): THREE.BufferGeometry {
  switch (kind) {
    case "hub": return new THREE.IcosahedronGeometry(0.78, 1);
    case "person": return new THREE.SphereGeometry(0.36, 28, 20);
    case "agent": return new THREE.DodecahedronGeometry(0.42, 0);
    case "checkpoint": return new THREE.TorusGeometry(0.5, 0.09, 14, 40);
    case "data": return new THREE.BoxGeometry(0.62, 0.07, 0.44);
    default: return new THREE.BoxGeometry(0.5, 0.3, 0.5);
  }
}

export function AgentWorld({ nodes, edges, pulses, selected, onSelect, tones = {}, active = [], absent = [], emphasis, currentId, className }: Props) {
  const wrap = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const labels = useRef<Map<string, HTMLButtonElement>>(new Map());
  const [failed, setFailed] = useState(false);
  const [paused, setPaused] = useState(false);
  const [listView, setListView] = useState(false);
  const live = useRef({ selected, tones, active, absent, emphasis, pulses, paused, reset: 0 });
  live.current = { ...live.current, selected, tones, active, absent, emphasis, pulses, paused };
  const nodeKey = useMemo(() => nodes.map((n) => n.id).join("|"), [nodes]);

  useEffect(() => {
    const el = wrap.current;
    const cv = canvas.current;
    if (!el || !cv) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ canvas: cv, antialias: true, alpha: true });
    } catch {
      setFailed(true);
      return;
    }
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    const scene = new THREE.Scene();
    scene.fog = new THREE.Fog(0x10100f, 22, 44);
    const camera = new THREE.PerspectiveCamera(40, 1, 0.1, 100);
    const HOME = new THREE.Vector3(0, 13.2, 17.2);
    camera.position.copy(HOME);
    const controls = new OrbitControls(camera, cv);
    controls.enablePan = false;
    controls.enableDamping = !reduce;
    controls.minDistance = 8;
    controls.maxDistance = 30;
    controls.maxPolarAngle = 1.42;
    const CENTER = new THREE.Vector3(0, 0.4, -0.9);
    controls.target.copy(CENTER);
    controls.autoRotateSpeed = 0.32;
    let touched = false; // stop the idle orbit as soon as someone takes the camera
    controls.addEventListener("start", () => { touched = true; });
    const targetGoal = CENTER.clone();

    scene.add(new THREE.HemisphereLight(0xf3ead8, 0x15140f, 1.15));
    const key = new THREE.DirectionalLight(0xfff1d6, 1.5);
    key.position.set(6, 12, 8);
    scene.add(key);

    // Ground: three thin orbits and a faint polar grid.
    const grid = new THREE.PolarGridHelper(11.5, 24, 1, 96, 0x2a2924, 0x24231f);
    (grid.material as THREE.Material).transparent = true;
    (grid.material as THREE.Material).opacity = 0.55;
    scene.add(grid);
    for (const r of [3.3, 6.4, 9.6]) {
      const pts = Array.from({ length: 129 }, (_, i) => new THREE.Vector3(Math.cos((i / 128) * Math.PI * 2) * r, 0.01, Math.sin((i / 128) * Math.PI * 2) * r));
      scene.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints(pts), new THREE.LineBasicMaterial({ color: 0x4a4841, transparent: true, opacity: 0.7 })));
    }
    const table = new THREE.Mesh(new THREE.CylinderGeometry(1.7, 1.7, 0.06, 64), new THREE.MeshStandardMaterial({ color: 0x24231f, roughness: 0.95 }));
    table.position.y = 0.03;
    scene.add(table);

    // A soft pool of light under the house agent.
    const gc = document.createElement("canvas");
    gc.width = gc.height = 256;
    const g2 = gc.getContext("2d");
    if (g2) {
      const grad = g2.createRadialGradient(128, 128, 0, 128, 128, 128);
      grad.addColorStop(0, "rgba(239,233,220,0.34)");
      grad.addColorStop(0.45, "rgba(217,164,65,0.10)");
      grad.addColorStop(1, "rgba(0,0,0,0)");
      g2.fillStyle = grad;
      g2.fillRect(0, 0, 256, 256);
    }
    const glowTex = new THREE.CanvasTexture(gc);
    const glow = new THREE.Mesh(new THREE.PlaneGeometry(9, 9), new THREE.MeshBasicMaterial({ map: glowTex, transparent: true, depthWrite: false, blending: THREE.AdditiveBlending }));
    glow.rotation.x = -Math.PI / 2;
    glow.position.y = 0.07;
    scene.add(glow);

    type Built = { node: WorldNode; pos: THREE.Vector3; mesh: THREE.Mesh; mat: THREE.MeshStandardMaterial; base: THREE.Color; sat?: THREE.Mesh };
    const built = new Map<string, Built>();
    const disposables: { dispose: () => void }[] = [];
    for (const n of nodes) {
      const pos = place(n);
      const base = new THREE.Color(KIND_COLOR[n.kind]);
      const mat = new THREE.MeshStandardMaterial({ color: base, roughness: 0.82, metalness: 0.04, emissive: base, emissiveIntensity: 0.12, transparent: true, flatShading: n.kind === "agent" || n.kind === "hub" });
      const geo = geometryFor(n.kind);
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.copy(pos);
      scene.add(mesh);
      disposables.push(geo, mat);
      // A stem ties each node to the ground so depth reads clearly.
      const stem = new THREE.Line(new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(pos.x, 0.02, pos.z), pos]), new THREE.LineBasicMaterial({ color: 0x4a4841, transparent: true, opacity: 0.6 }));
      scene.add(stem);
      const b: Built = { node: n, pos, mesh, mat, base };
      if (n.kind === "person") {
        // Each flatmate's own agent: a small satellite.
        const sm = new THREE.MeshStandardMaterial({ color: 0x8fb08a, emissive: 0x8fb08a, emissiveIntensity: 0.35, roughness: 0.7, transparent: true });
        const sg = new THREE.OctahedronGeometry(0.13, 0);
        b.sat = new THREE.Mesh(sg, sm);
        scene.add(b.sat);
        disposables.push(sg, sm);
      }
      built.set(n.id, b);
    }

    const edgeLines: { line: THREE.Line; mat: THREE.LineBasicMaterial; from: string; to: string }[] = [];
    for (const e of edges) {
      const a = built.get(e.from), b = built.get(e.to);
      if (!a || !b) continue;
      const mat = new THREE.LineBasicMaterial({ color: 0x6a6659, transparent: true, opacity: 0.38 });
      const line = new THREE.Line(new THREE.BufferGeometry().setFromPoints(arc(a.pos, b.pos).getPoints(36)), mat);
      scene.add(line);
      edgeLines.push({ line, mat, from: e.from, to: e.to });
    }

    const ring = new THREE.Mesh(new THREE.TorusGeometry(0.78, 0.025, 8, 64), new THREE.MeshBasicMaterial({ color: 0xd9a441 }));
    ring.rotation.x = Math.PI / 2;
    ring.visible = false;
    scene.add(ring);

    // Pulses: small beads that replay recorded events along their route.
    const beadGeo = new THREE.SphereGeometry(0.1, 12, 10);
    const beads: { parts: THREE.Mesh[]; curve: THREE.QuadraticBezierCurve3; offset: number }[] = [];
    const flash = new Map<string, number>(); // node id -> time it last received something new
    let pulseKey: string | null = null;
    const syncPulses = (t: number) => {
      const k = live.current.pulses.map((p) => `${p.from}>${p.to}:${p.tone}`).join("|");
      if (k === pulseKey) return;
      const first = pulseKey === null;
      pulseKey = k;
      for (const b of beads) for (const m of b.parts) { scene.remove(m); (m.material as THREE.Material).dispose(); }
      beads.length = 0;
      live.current.pulses.slice(0, 10).forEach((p, i) => {
        const a = built.get(p.from), b = built.get(p.to);
        if (!a || !b) return;
        // A lead bead and two fainter ones behind it read as a short trail.
        const parts = [1, 0.66, 0.4].map((scale, j) => {
          const m = new THREE.Mesh(beadGeo, new THREE.MeshBasicMaterial({ color: TONE_HEX[p.tone], transparent: true, opacity: 1 - j * 0.32 }));
          m.scale.setScalar(scale);
          scene.add(m);
          return m;
        });
        beads.push({ parts, curve: arc(a.pos, b.pos), offset: i * 0.37 });
      });
      // When something new is recorded, its sender and receiver light up briefly.
      const newest = live.current.pulses[0];
      if (!first && newest) { flash.set(newest.from, t); flash.set(newest.to, t); }
    };

    const size = () => {
      const { clientWidth: w, clientHeight: h } = el;
      if (!w || !h) return;
      renderer.setSize(w, h, false);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      if (!homed) {
        // Start from the default view once the canvas has its real size.
        homed = true;
        camera.position.copy(HOME);
        controls.target.copy(CENTER);
        controls.update();
      }
    };
    let homed = false;
    const ro = new ResizeObserver(size);
    const settle = window.setTimeout(() => { homed = false; size(); }, 350);
    ro.observe(el);
    size();

    const v = new THREE.Vector3();
    const RANK: Record<WorldNode["kind"], number> = { hub: 0, person: 1, checkpoint: 2, agent: 3, tool: 4, data: 5 };
    const placing: { el: HTMLButtonElement; x: number; y: number; below: boolean; rank: number; depth: number }[] = [];
    const clock = new THREE.Clock();
    let raf = 0;
    let lastReset = live.current.reset;
    const frame = () => {
      raf = requestAnimationFrame(frame);
      const t = clock.getElapsedTime();
      const s = live.current;
      const moving = !s.paused && !reduce;
      syncPulses(t);
      if (s.reset !== lastReset) {
        lastReset = s.reset;
        camera.position.copy(HOME);
        targetGoal.copy(CENTER);
        touched = false;
      }
      const sel = s.selected ? built.get(s.selected) : undefined;
      targetGoal.lerp(sel ? new THREE.Vector3(sel.pos.x * 0.55, 0.5, sel.pos.z * 0.55) : CENTER, 0.08);
      controls.autoRotate = moving && !touched && !sel;
      controls.target.lerp(targetGoal, reduce ? 1 : 0.12);
      controls.update();

      for (const b of built.values()) {
        const id = b.node.id;
        const dim = (s.emphasis && !s.emphasis.includes(id)) || s.absent.includes(id);
        const tone = s.tones[id];
        const isActive = s.active.includes(id);
        b.mat.color.copy(tone ? new THREE.Color(TONE_HEX[tone]) : b.base);
        b.mat.emissive.copy(b.mat.color);
        b.mat.opacity = dim ? 0.2 : 1;
        const since = t - (flash.get(id) ?? -10);
        const burst = since < 1.4 && !reduce ? (1 - since / 1.4) * 0.9 : 0;
        b.mat.emissiveIntensity = burst + (isActive ? 0.38 + (moving ? Math.sin(t * 2.4 + b.pos.x) * 0.16 : 0) : 0.1) + (id === s.selected ? 0.25 : 0);
        const k = id === s.selected ? 1.18 : isActive && moving ? 1 + Math.sin(t * 2.4 + b.pos.z) * 0.04 : 1;
        b.mesh.scale.setScalar(k);
        if (b.node.kind === "hub" && moving) b.mesh.rotation.y = t * 0.18;
        if (b.node.kind === "checkpoint") b.mesh.lookAt(0, b.pos.y, 0);
        if (b.sat) {
          const a = moving ? t * 0.9 + b.pos.x : b.pos.x;
          b.sat.position.set(b.pos.x + Math.cos(a) * 0.62, b.pos.y + 0.28 + Math.sin(a * 1.3) * 0.06, b.pos.z + Math.sin(a) * 0.62);
          (b.sat.material as THREE.MeshStandardMaterial).opacity = dim ? 0.15 : 1;
        }
        // Labels follow their node; systems sit under theirs, everything else above.
        const lab = labels.current.get(id);
        if (lab) {
          const below = b.node.kind === "tool" || b.node.kind === "data";
          v.copy(b.pos).setY(b.pos.y + (b.node.kind === "hub" ? 1.05 : below ? -0.42 : 0.62)).project(camera);
          lab.style.display = v.z < 1 ? "" : "none";
          lab.style.opacity = dim ? "0.3" : "1";
          if (v.z < 1) placing.push({ el: lab, x: ((v.x + 1) / 2) * el.clientWidth, y: ((1 - v.y) / 2) * el.clientHeight, below, rank: RANK[b.node.kind] + (id === s.selected ? -10 : 0), depth: v.z });
        }
      }
      // Keep labels from sitting on top of each other: the more important one keeps its place.
      placing.sort((p, q) => p.rank - q.rank);
      const taken: { l: number; r: number; t: number; b: number }[] = [];
      for (const p of placing) {
        const w = p.el.offsetWidth || 90, h = p.el.offsetHeight || 24;
        let top = p.below ? p.y : p.y - h;
        const clash = () => {
          const box = { l: p.x - w / 2, r: p.x + w / 2, t: top, b: top + h };
          return taken.some((o) => box.l < o.r + 4 && box.r > o.l - 4 && box.t < o.b + 2 && box.b > o.t - 2);
        };
        // One small nudge keeps a label next to its node; a system label that still collides steps aside.
        if (clash()) top += (p.below ? 1 : -1) * (h + 3);
        if (clash() && p.rank >= 4) { p.el.style.display = "none"; continue; }
        taken.push({ l: p.x - w / 2, r: p.x + w / 2, t: top, b: top + h });
        p.el.style.transform = `translate(${Math.round(p.x - w / 2)}px, ${Math.round(top)}px)`;
        p.el.style.zIndex = String(1000 - Math.round(p.depth * 1000));
      }
      placing.length = 0;
      for (const e of edgeLines) {
        const dim = s.emphasis && !(s.emphasis.includes(e.from) && s.emphasis.includes(e.to));
        const hot = s.selected && (e.from === s.selected || e.to === s.selected);
        e.mat.opacity = dim ? 0.07 : hot ? 0.95 : 0.38;
        e.mat.color.set(hot ? 0xd9a441 : 0x6a6659);
      }
      ring.visible = !!sel;
      if (sel) { ring.position.set(sel.pos.x, 0.03, sel.pos.z); ring.scale.setScalar(sel.node.kind === "hub" ? 1.5 : 1); }
      for (const b of beads) {
        const p = (t * 0.34 + b.offset) % 1.6; // travel for 1, then rest
        b.parts.forEach((m, j) => {
          const q = p - j * 0.045;
          m.visible = moving && q >= 0 && q <= 1;
          if (m.visible) m.position.copy(b.curve.getPoint(q));
        });
      }
      renderer.render(scene, camera);
    };
    frame();

    return () => {
      cancelAnimationFrame(raf);
      window.clearTimeout(settle);
      ro.disconnect();
      controls.dispose();
      for (const d of disposables) d.dispose();
      beadGeo.dispose();
      glowTex.dispose();
      renderer.dispose();
    };
    // The scene is rebuilt only when the set of nodes changes; everything else is read live.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nodeKey, failed]);

  const groups: [string, WorldNode["kind"][]][] = [["People", ["person"]], ["Agents", ["hub", "agent"]], ["Human checkpoint", ["checkpoint"]], ["Systems", ["tool", "data"]]];
  const list = (
    <div className="grid gap-4 p-4 sm:grid-cols-2 lg:grid-cols-4">
      {groups.map(([title, kinds]) => (
        <div key={title}>
          <h3 className="mb-2 text-xs font-semibold uppercase tracking-widest text-stone-500">{title}</h3>
          <ul className="space-y-1">
            {nodes.filter((n) => kinds.includes(n.kind)).map((n) => (
              <li key={n.id}>
                <button onClick={() => onSelect(n.id)} aria-pressed={selected === n.id} className={`w-full rounded-lg border px-3 py-2 text-left text-sm ${selected === n.id ? "border-ember bg-raised" : "border-line hover:bg-raised"}`}>
                  <span className="font-medium">{n.label}</span>
                  <span className="block text-xs text-stone-500">{n.role}</span>
                </button>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );

  return (
    <div className={`relative overflow-hidden rounded-2xl border border-line bg-[radial-gradient(ellipse_at_50%_35%,#22211d_0%,#141412_62%,#10100f_100%)] ${className ?? ""}`}>
      <div className="absolute left-3 top-3 z-[1100] flex flex-wrap gap-1.5">
        {!failed && !listView && (
          <>
            <button onClick={() => { live.current.reset += 1; onSelect(null); }} className="rounded-lg border border-line bg-ink/80 px-2.5 py-1 text-xs font-medium backdrop-blur hover:bg-raised">Reset view</button>
            <button disabled={!currentId} onClick={() => currentId && onSelect(currentId)} className="rounded-lg border border-line bg-ink/80 px-2.5 py-1 text-xs font-medium backdrop-blur hover:bg-raised disabled:opacity-40">Focus on current activity</button>
            <button onClick={() => setPaused((p) => !p)} aria-pressed={paused} className="rounded-lg border border-line bg-ink/80 px-2.5 py-1 text-xs font-medium backdrop-blur hover:bg-raised">{paused ? "Resume motion" : "Pause motion"}</button>
          </>
        )}
        {!failed && <button onClick={() => setListView((l) => !l)} aria-pressed={listView} className="rounded-lg border border-line bg-ink/80 px-2.5 py-1 text-xs font-medium backdrop-blur hover:bg-raised">{listView ? "3D view" : "List view"}</button>}
      </div>
      {failed ? (
        <div className="pt-12">
          <p className="px-4 text-sm text-stone-500">3D view is unavailable in this browser, so here is the same system as a list.</p>
          {list}
        </div>
      ) : (
        <>
          <div ref={wrap} className={listView ? "hidden" : "absolute inset-0"}>
            <canvas ref={canvas} className="h-full w-full" aria-hidden="true" />
            <div className="pointer-events-none absolute inset-0">
              {nodes.map((n) => (
                <button
                  key={n.id}
                  ref={(el) => { if (el) labels.current.set(n.id, el); else labels.current.delete(n.id); }}
                  onClick={() => onSelect(selected === n.id ? null : n.id)}
                  aria-pressed={selected === n.id}
                  aria-label={`${n.label}: ${n.role}`}
                  title={n.role}
                  className={`pointer-events-auto absolute left-0 top-0 whitespace-nowrap rounded-full border px-2.5 py-1 text-xs font-medium backdrop-blur transition-colors ${
                    selected === n.id ? "border-ember bg-ink text-ivory" : n.kind === "tool" || n.kind === "data" ? "border-line bg-ink/60 text-stone-500 hover:text-ivory" : "border-line bg-ink/75 text-stone-800 hover:border-stone-400"
                  }`}
                >
                  <span className="mr-1.5 inline-block h-1.5 w-1.5 rounded-full align-middle" style={{ background: tones[n.id] ? TONE_HEX[tones[n.id]] : KIND_COLOR[n.kind] }} />
                  {n.label}
                </button>
              ))}
            </div>
          </div>
          {listView && <div className="pt-12">{list}</div>}
          {!listView && (
            <ul className="pointer-events-none absolute bottom-3 right-3 z-[1100] hidden gap-3 rounded-lg border border-line bg-ink/70 px-3 py-1.5 text-[11px] text-stone-600 backdrop-blur sm:flex" aria-label="Legend">
              <li><span className="mr-1.5 inline-block h-2 w-2 rounded-full align-middle" style={{ background: KIND_COLOR.person }} />People</li>
              <li><span className="mr-1.5 inline-block h-2 w-2 rotate-45 align-middle" style={{ background: KIND_COLOR.agent }} />Agents</li>
              <li><span className="mr-1.5 inline-block h-2 w-2 rounded-full border-2 align-middle" style={{ borderColor: KIND_COLOR.checkpoint }} />Human check</li>
              <li><span className="mr-1.5 inline-block h-2 w-2 align-middle" style={{ background: KIND_COLOR.tool }} />Systems</li>
            </ul>
          )}
          {!listView && (
            <p className="pointer-events-none absolute bottom-3 left-3 z-[1100] text-[11px] text-stone-500">
              Drag to orbit · scroll to zoom · {paused ? "motion paused" : "beads replay the latest recorded events"}
            </p>
          )}
        </>
      )}
    </div>
  );
}
