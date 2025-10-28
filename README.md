# krets

My SPICE circuit simulator called `krets` written in rust.

I am doing this project to learn rust.

## Usage

Create a spice circuit description file, e.g `low_pass_filter.cir`:

```cir
V1 in 0 1 AC 1
R1 in out 1000
C1 out 0 1u
```

Then create a krets configuration file, e.g. `krets.toml`:

```toml
circuit_path = "low_pass_filter.cir"

[analysis.ac]
fstart = 1
fstop = 1000
npoints = 100
```

Then run krets with the configuration file:

```bash
krets krets.toml
```

## Supported components

- [ ] BJT
- [x] Capacitor
- [x] Current Source
- [ ] Diode
- [x] Inductor
- [ ] Mosfet
- [ ] Mutual Inductor
- [x] Resistor
- [x] Voltage Source
- [ ] Subcircuit definitions

## Supported analyses

- [x] OP
- [x] DC
- [x] AC
- [x] Transient
- [ ] Harmonic Balance
- [ ] S parameter analysis

## Order of things TODO

- Add the component VCVS (Voltage Controlled Voltage Source)
- Add the component VCCS (Voltage Controlled Current Source)
- Add the component CCVS (Current Controlled Voltage Source)
- Add the component CCCS (Current Controlled Current Source)
- Add transient analyses
