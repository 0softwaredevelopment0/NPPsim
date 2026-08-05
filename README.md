# NPP-SIM — Nuclear Power Plant Control Simulator (VVER-1000)

![Development status](https://img.shields.io/badge/status-Alpha-red)

**A desktop Java (Swing) application. Portable version:**
copy the whole folder anywhere and run `NPPsim.bat`.

## Folder contents

```
NPPsim/
├── NPPsim.bat     ← portable launcher (double-click)
├── NPPsim.jar     ← executable program (small, standard Java only)
├── src/           ← source code (Reactor.java, NPPGui.java, NPPsim.java, PhysicsTest.java)
├── web-version/   ← bonus: the previous web version (index.html, _test.js)
└── README.md
```

The program is written in Java and uses only the standard library
(`javax.swing`). It requires no internet connection and no installation —
only **Java 8+** (JRE/JDK) is needed. If Java is not installed, download it:
https://adoptium.net (or place a `jre` folder next to `NPPsim.bat` — the
launcher will find it first).

## Launch

- Double-click `NPPsim.bat`.
- Or in a console: `java -jar NPPsim.jar`
- Version check: `java -jar NPPsim.jar --version`
- Physics self-test: `java -jar NPPsim.jar --selftest` (or `java PhysicsTest`)

## What is simulated

- **Reactor**: point kinetics of power, **CPS control rods with individual
  withdrawal** (rod selection, % withdrawal, "Select all" button), 3 speeds
  + ↑/↓ keys, negative temperature coefficient,
  **xenon poisoning** (iodine pit), **boron regulation**,
  APR — automatic power regulator (setpoint in %).
- **Primary circuit**: **4 main circulation pumps MCP-1..4 with separate
  ON/OFF switches** (flow depends on the number running: 4 — 100%,
  3 — 75%), pressurizer with **charging/letdown**, relief valve,
  leaks.
- **Secondary circuit**: steam generator, **BRU-A with AUT/MAN/OFF modes**
  and a manual open button, atmospheric dump (GPD), **feed pumps FP-1/FP-2
  and emergency pump EFP** (switched on separately), SG level.
- **Turbine unit**: steam inlet, **generator breaker**, grid frequency, MW(e).
- **Accidents and protections**: AZ (button/key **A** and automatic logic),
  MCP failure, primary-circuit leak (severe accident), steam-line rupture,
  feed pump failure, grid disconnection, dropped assembly, spurious AZ,
  random event.
- **Fuel temperature and melting**: each of the 9×9 fuel assemblies has its
  own FUEL temperature (center hotter than edges, depends on power and heat
  removal). Above **2800 °C** the fuel melts: the assembly darkens and is
  lost — a radiation accident. Keep the temperature under control with rods,
  boron, MCPs and feed water.
- **Reactor overview**: the core "as a ball" (top view) — 9×9 fuel assemblies
  (FA), each with its own fuel temperature and color (scale 300–2800 °C),
  melted assemblies are highlighted; radial profile and a melted-assembly
  counter.
- **Operator tasks**: every scenario has goals with checks — use ALL controls
  (rods, boron, MCPs, feed water, turbine, generator) to pass the shift.

## Scenarios and tasks

Each scenario is an operator shift with goals (the "OPERATOR TASKS" panel):

| Button | Task |
|---|---|
| Run 100% | keep power at 90–110% and pressure at 15–18 MPa for 2 min, no AZ, no damage |
| Unit startup | raise power >50%, start the turbine, reach ≥950 MW(e) |
| Shutdown | lower power <10%, insert rods to ≥95% |

Goals are marked ✔/✘ as the shift progresses; after an accident urgent tasks
are added (fix the leak, restore flow, connect the generator…).
The shift is completed successfully when all goals are closed.

## Controls

Controls look like a real main control room: switches (rockers) with lamps,
mode selectors and buttons — not a "sandbox" of sliders.

- **MCP-1..4** — separate ON/OFF switches (green lamp = running).
  Flow = number of pumps ÷ 4. Rules: at power >75% you must not lose MCPs
  (AZ), switching off two MCPs — AZ. At low power (<10%) they may be stopped.
- **FP-1, FP-2** (50% each) and **EFP** (emergency, 30%) — separate switches.
  Feed water = sum of running pumps; watch the level when the SG overheats.
- **BRU-A**: mode selector **AUT / MAN / OFF** + **"Open"** button (in MAN).
  AUT dumps steam to the condenser at 7.59 MPa; OFF leads to the atmospheric
  dump (GPD).
- **Pressurizer charging**: pressurizer switch. OFF — the pressure slowly
  bleeds down, don't forget to switch it back.
- **Generator** — grid switch; **turbine** — steam inlet slider.
- **Rods — "% WITHDRAWAL"**: rods are drawn as circles right on the reactor
  overview (3×3 positions, under each — its % withdrawal). Click a rod to
  select it (amber ring), or use the **"Select all"** / "Clear selection"
  button. Enter the % withdrawal and press **"Apply"**. 0% = rod in the core,
  100% = withdrawn (power rises). For manual control switch off the APR.
- **Rods — buttons**: ▲/▼ (hold) or ↑/↓ arrows; space — stop; **A** — AZ.
- **APR**: toggle + power setpoint, %.
- **Boron** — slider (boron regulation — a continuous quantity).
- **AZ**: two red key buttons **AZ-1** and **AZ-2** (duplicated).
- **Events** — accident training buttons; **Time acceleration**: 1× / 10× / 60× / 600×.
- **Left view**: tabs "Reactor overview" (core with assemblies) and "Unit diagram".

A reminder about real VVER-1000 rules: switching off an MCP at power, loss of
feed water and pressurizer charging switched off are emergency modes — the
protection will act on its own, but it is better to manage proactively.

## Emergency protection (AZ) — three states

| State | Meaning |
|---|---|
| AZ: ON | automatic logic active: in an accident it will trip on its own |
| AZ: TRIPPED | protection has tripped and **switched off** — automatic protection is disabled until re-armed |
| AZ: OFF | the operator switched off the automatic protection with a button — protections WILL NOT trip! |

- After an AZ trip you need to **"Re-arm AZ"** — until then the automatic
  logic stays silent.
- "Disable AZ" is a high-risk mode for training manual operation: the
  automatic logic does not interfere, and the failed protection is logged
  (ACCIDENT!).
- The **A** button/key (manual AZ) always works.

## Protections (trip automatically when AZ: ON)

AZ: on core temperature, power exceedance (>115%), high/low primary-circuit
pressure, loss of flow, **two MCPs switched off**, **an MCP switched off at
power >75%**, low SG level. Turbine trip: low SG level, steam pressure drop.

## Fuel melting

Assembly fuel melts above **2800 °C** (typical cases: reactivity excursion
with AZ disabled, loss of cooling). Signs of approach: yellow-white assembly
coloring, "Fuel temperature limit" alarm (>2600 °C).
A melted assembly is black with a red border; melting even one assembly is a
radiation accident and the unit is lost. Keep the fuel temperature under
control!

## Operator tips

- During shutdown watch the xenon: ~40 minutes after the power drop is the
  "iodine pit" — an immediate restart is impossible, adjust the boron.
- Switch off the APR for manual operation (the "Startup" scenario).
- Pass shifts by the tasks: first "Run 100%", then "Startup" and "Shutdown".
- Disabling AZ is a last-resort training measure: make sure you understand
  the consequences.

## Building from source

```
cd src
javac -encoding UTF-8 -d ../out *.java
jar cfe ../NPPsim.jar NPPsim -C ../out .
```

## Tests

`java -jar NPPsim.jar --selftest` — 39 physics and logic checks (100% stability,
AZ, iodine pit, LOCA, flow/feed loss, excursion, steam-line rupture, recovery,
AZ states, task engine, individual rods, fuel temperature, melting, power
protection, MCP 4×ON/OFF, BRU-A AUT/MAN/OFF and GPD, pressurizer charging).
All tests pass.
