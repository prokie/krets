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

The conductance of the diode is $G_D$ and is given by the derivative of the Shockley diode equation with respect to the voltage:

$ G_D = frac(d I_D, d V_D) = frac(I_S, n V_T) e^frac(V_D, n V_T) $ <shockley-diode-conductance>



=== Voltage Source

In the conductance matrix the stamps for a voltage source are given by:

If the positive terminal is connect to node `i` and the node is not grounded, the stamp is: 1


== Analyses


=== DC

During DC analysis, the circuit is analyzed under steady-state conditions with all capacitors treated as open circuits and all inductors treated as short circuits.




==== Diode IV Curve

#let v1 = 1
#let r1 = 1000
$cases(V_1 = 1, R_1 = 1000)$

#figure(caption: "Diode IV Curve", zap.canvas({
  import zap: *
  vsource("v1", (0, 0), (0, 4), label: $V_1$)
  resistor("r1", (0, 4), (3, 4), label: $R_1$)
  diode("d1", (3, 4), (3, 0), label: $R_1$)
  ground("gnd", (0, 0))
  wire((0, 0), (3, 0))
}))

$ I_D = frac(V_("out") - V_1, R_1) $
$ I_D = I_S (e^frac(V_D, n V_T) - 1) $


#let initial_guess = 0.5
We guess the diode voltage $V_D=#initial_guess$ and calculate the diode current $I_D$ using the Shockley diode equation.

$ I_D = #shockley_diode_current(initial_guess) $

Then we solve for $V_"out"$:
#let vout = shockley_diode_current(initial_guess) * r1 + v1



$V_"out" = I_D R_1 + V_"in" = #vout$



#set heading(numbering: "A.1")
#counter(heading).update(0)



= Appendix
#include "appendix.typ"


