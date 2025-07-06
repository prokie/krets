#import "@preview/zap:0.2.0"
#import "@preview/cetz:0.4.0"
#import "@preview/cetz-plot:0.1.2"
#import "constants.typ": shockley_diode_conductance, shockley_diode_current

#set heading(numbering: "1.")
#set math.equation(numbering: "(1)")



= Krets





== Elements

=== Diode

The diode is modeled as a nonlinear element with a current-voltage relationship defined by the Shockley diode equation:

$ I_D = I_S (e^frac(V_D, n V_T) - 1) $ <shockley-diode-equation>

Where
$I_D$ is the diode current,
$I_S$ is the reverse saturation current,
$V_D$ is the voltage across the diode,
$V_T$ is the thermal voltage, and,
$n$ is the ideality factor, also known as the quality factor, emission coefficient, or the material constant.

#let hej = ()
#for value in range(0, 74).map(x => x / 100) {
  hej.push((value, shockley_diode_current(value)))
}

#figure(caption: "Diode IV Curve", cetz.canvas({
  import cetz.draw: *
  import cetz-plot: *
  plot.plot(size: (5, 5), x-tick-step: .2, y-tick-step: .5, x-grid: true, y-grid: true, plot.add(hej))
}))

#shockley_diode_current(0)
#shockley_diode_conductance(0)

=== Voltage Source

In the conductance matrix the stamps for a voltage source are given by:

If the positive terminal is connect to node `i` and the node is not grounded, the stamp is: 1


== Analyses


=== DC

During DC analysis, the circuit is analyzed under steady-state conditions with all capacitors treated as open circuits and all inductors treated as short circuits.




==== Diode IV Curve


#figure(caption: "Diode IV Curve", zap.canvas({
  import zap: *
  vsource("v1", (0, 0), (0, 4), label: "V1")
  resistor("r1", (0, 4), (3, 4), label: "R1")
  diode("d1", (3, 4), (3, 0), label: "R1")
  ground("gnd", (0, 0))
  wire((0, 0), (3, 0))
}))




$
  mat(
    0, 1, 0;
    1, 0.001, -0.001;
    0, -0.001, 0.001;
  )
  mat(
    "I(V1)";
    "V(in)";
    "V(out)";
  ) =
  mat(
    0;
    1;
    1;
  )
$

#set heading(numbering: "A.1")
#counter(heading).update(0)



= Appendix
#include "appendix.typ"


