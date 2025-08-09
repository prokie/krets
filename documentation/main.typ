#import "@preview/zap:0.2.1"
#import "@preview/cetz:0.4.0"
#import "@preview/cetz-plot:0.1.2"
#import "constants.typ": shockley_diode_conductance, shockley_diode_current

#set heading(numbering: "1.")
#set math.equation(numbering: "(1)")



= Krets





== Elements

=== BJT

=== Capacitor

Element stamps for a capacitor in the conductance matrix in group 1:

$
  mat(
    delim: #none,
    , , v^+, , v^-, , |, "RHS";
    , , dots.v, , dots.v, , |, dots.v;
    v^+, dots, +C, dots, -C, dots, |, dots.v;
    v^-, dots, -C, dots, +C, dots, |, dots.v;
    , , dots.v, , dots.v, , |, dots.v;
  )
$

Element stamps for a capacitor in the conductance matrix in group 2:

$
  mat(
    delim: #none,
    , , v^+, , v^-, , i_C, , |, "RHS";
    , , dots.v, , dots.v, , dots.v, , |, ;
    v^+, dots, dots, dots, dots, dots, dots, dots, |, ;
    v^-, dots, dots, dots, dots, dots, dots, dots, |, ;
    i_C, dots, -C, dots, C, dots, + 1, dots, |, ;
    , , dots.v, , dots.v, , dots.v, , |, ;
  )
$



=== Current Source



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
  plot.plot(
    size: (5, 5),
    x-tick-step: .2,
    y-tick-step: .5,
    x-grid: true,
    y-grid: true,
    x-label: $V_D$,
    y-label: $I_D$,
    plot.add(hej),
  )
}))

The conductance of the diode is $G_D$ and is given by the derivative of the Shockley diode equation with respect to the voltage:

$ G_D = frac(d I_D, d V_D) = frac(I_S, n V_T) e^frac(V_D, n V_T) $ <shockley-diode-conductance>

The companion model for the diode can be represented as a current source in parallel with a conductance.


$ I_"eq" = I_D - G_"eq" V_D $

#figure(caption: "Diode Companion model", zap.canvas({
  import zap: *
  wire((0, 0), (3, 0))
  wire((3, 0), (3, -0.5))
  wire((2, -0.5), (4, -0.5))
  node("n1", (3, -0.5))
  resistor("r1", (2, -0.5), (2, -3), label: $frac(1, G_"eq")$)
  isource("i1", (4, -0.5), (4, -3), label: $I_"eq"$)
  wire((2, -3), (4, -3))
  wire((3, -3), (3, -3.5))
  node("n2", (3, -3))
  wire((0, -3.5), (3, -3.5))
}))


The element stamps for the diode in the conductance matrix are given by:

$
  mat(
    delim: #none,
    , , v^+, , v^-, , |, "RHS";
    , , dots.v, , dots.v, , |, dots.v;
    n^+, dots, +G_"eq", dots, -G_"eq", dots, |, -I_"eq";
    n^-, dots, -G_"eq", dots, +G_"eq", dots, |, +I_"eq";
    , , dots.v, , dots.v, , |, dots.v;
  )
$




=== Inductor

=== Mosfet

=== Resistor


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
  resistor("r1", (0, 4), (3, 4), label: $R$)
  diode("d1", (3, 4), (3, 0), label: $D$)
  ground("gnd", (0, 0))
  wire((0, 0), (3, 0))
}))


Lets build the conductance matrix for this circuit.


$
  mat(
    0, 1, 0;
    1, frac(1, R), -frac(1, R);
    0, -frac(1, R), frac(1, R) + G_D
  )
  mat(delim: "|", I(V_1); V_"in"; V_"out") = mat(1; 0; I_D)
$



#set heading(numbering: "A.1")
#counter(heading).update(0)



= Appendix
#include "appendix.typ"


