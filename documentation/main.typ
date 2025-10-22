#import "@preview/zap:0.4.0"
#import "@preview/cetz:0.4.2"
#import "@preview/cetz-plot:0.1.3"
#import "constants.typ": shockley_diode_conductance, shockley_diode_current

#set heading(numbering: "1.")
#set math.equation(numbering: "(1)")
#show figure.where(
  kind: table,
): set figure.caption(position: top)
#show math.equation: set text(size: 11pt)

= Krets





== Elements


=== Resistor

#figure(caption: "A resistor.", zap.circuit({
  import zap: *
  set-style(zap: (variant: "ieee"))
  node("n1", (1, 0), label: (content: "n+", anchor: "west"))
  node("n2", (3, 0), label: (content: "n-", anchor: "east"))

  resistor("r1", (1, 0), (3, 0))
  draw.line((1.1, 0.5), (2.9, 0.5), mark: (end: ">", fill: black))
  draw.content((2, 0.8), $i_R$)
}))



The current through a resistor is given by Ohm's law:

$ i_R = frac(v^+ - v^-, R) = G(v^+ - v^-) $

To get the element stamps for the resistor in the conductance matrix, we use Kirchhoff's Current Law (KCL) at nodes n+ and n-.

At node n+:
$ i_(n+) = i_R = G(v^+ - v^-) $
At node n-:
$ i_(n-) = -i_R = -G(v^+ - v^-) $

This leads to the following conductance matrix stamps:

#figure(
  table(
    columns: 4,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $v^+$, $v^-$, "RHS",
      table.hline(),
      $v^+$, $+G$, $-G$, $$,
      $v^-$, $-G$, $+G$, $$,
      table.hline(),
    ),
  ),
  caption: [Element stamps for a resistor in the conductance matrix in group 1.],
)

To get the stamps in group 2, we introduce a current variable $i_R$ for the resistor:

$ v^+ - v^- = R i_R => v^+ - v^- - R i_R = 0 $

#figure(
  table(
    columns: 5,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $v^+$, $v^-$, $i_R$, "RHS",
      table.hline(),
      $v^+$, [], [], $+1$, [],
      $v^-$, [], [], $-1$, $$,
      $i_R$, $+1$, $-1$, $-R$, $$,
      table.hline(),
    ),
  ),
  caption: [Element stamps for a resistor in the conductance matrix in group 2.],
)



=== BJT

=== Capacitor


#figure(
  table(
    columns: 4,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $v^+$, $v^-$, "RHS",
      table.hline(),
      $v^+$, $+C$, $-C$, $$,
      $v^-$, $-C$, $+C$, $$,
      table.hline(),
    ),
  ),
  caption: [Element stamps for a capacitor in the conductance matrix in group 1.],
)



#figure(
  table(
    columns: 5,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $v^+$, $v^-$, $i_C$, "RHS",
      table.hline(),
      $v^+$, [], [], $$, [],
      $v^-$, [], [], $$, $$,
      $i_C$, $-C$, $+C$, $+1$, $$,
      table.hline(),
    ),
  ),
  caption: [Element stamps for a capacitor in the conductance matrix in group 2.],
)

#figure(caption: "Capacitor Companion model for Backwards Euler.", zap.circuit({
  import zap: *
  set-style(zap: (variant: "ieee"))

  wire((0, 0), (3, 0), i: $i_(n+1)$)
  wire((3, 0), (3, -0.5))
  wire((2, -0.5), (4, -0.5))
  node("n1", (3, -0.5))
  resistor("r1", (2, -0.5), (2, -3), label: $frac(h, C)$)
  isource("i1", (4, -3), (4, -0.5), label: (content: $frac(C, h)u_n$, anchor: "south"))
  wire((2, -3), (4, -3))
  wire((3, -3), (3, -3.5))
  node("n2", (3, -3))
  wire((0, -3.5), (3, -3.5))
  draw.line((1, -1.5), (1, -0.5), mark: (end: ">", fill: black))
  draw.line((1, -2), (1, -3), mark: (end: ">", fill: black))
  draw.content((1, -1.75), $u_(n+1)$)
  draw.content((1, -3.25), $-$)
  draw.content((1, -0.25), $+$)
}))


The dynamic element equation

$ i(t_(n+1)) = C(u(t_(n+1)))u'(t_(n+1)) $

using $i(t_(n+1)) approx i_(n+1)$ and $u(t_(n+1)) approx u_(n+1)$

$ i_(n+1) = C(u_(n+1))u'(t_(n+1)) approx C(u_(n+1)) (frac(u_(n+1) - u_n, h)) $

$ u_(n+1) = frac(h, C) i_(n+1) + u_n $

so $ G_(n+1) = frac(h, C) "and" u_n = u_(n+1) -G_(n+1)i_(n+1) $



=== Current Source

#figure(caption: "A current source.", zap.circuit({
  import zap: *
  set-style(zap: (variant: "ieee"))
  node("n1", (1, 0), label: (content: "n+", anchor: "west"))
  node("n2", (3, 0), label: (content: "n-", anchor: "east"))
  isource("i1", (1, 0), (3, 0), label: $i_S$)
}))

The current through an independent current source is given by:
$ i_S = I $
To get the element stamps for the independent current source in the conductance matrix, we use Kirchhoff's Current Law (KCL) at nodes n+ and n-.

At node n+:
$ i_(n+) = i_S = I $
At node n-:
$ i_(n-) = -i_S = -I $

This leads to the following conductance matrix stamps:

#figure(
  table(
    columns: 4,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $v^+$, $v^-$, "RHS",
      table.hline(),
      $v^+$, [], [], $+I$,
      $v^-$, [], [], $-I$,
      table.hline(),
    ),
  ),
  caption: [Element stamp for an independent current source in group 1],
)







#figure(
  table(
    columns: 5,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $v^+$, $v^-$, $i$, "RHS",
      table.hline(),
      $v^+$, [], [], $+1$, [],
      $v^-$, [], [], $-1$, $$,
      $i$, [], [], $+1$, $I$,
      table.hline(),
    ),
  ),
  caption: [Element stamp for an independent current source in group 2],
)

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

#figure(caption: "Diode Companion model", zap.circuit({
  import zap: *
  set-style(zap: (variant: "ieee"))
  wire((0, 0), (3, 0), i: $I_D$)
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


#figure(
  table(
    columns: 4,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $v^+$, $v^-$, "RHS",
      table.hline(),
      $v^+$, $+G_"eq"$, $-G_"eq"$, $-I_"eq"$,
      $v^-$, $-G_"eq"$, $+G_"eq"$, $+I_"eq"$,
      table.hline(),
    ),
  ),
  caption: [Element stamps for a diode in group 1.],
)





=== Inductor



The dynamic element equation

$ u(t_(n+1)) = L(i(t_(n+1)))i'(t_(n+1)) $

using $u(t_(n+1)) approx u_(n+1)$ and $i(t_(n+1)) approx i_(n+1)$


$ u_(n+1) = L(i_(n+1))i'(t_(n+1)) approx L(i_(n+1)) (frac(i_(n+1) - i_n, h)) $

$ i_(n+1) = frac(h, L)u_"n+1" + i_n $

$ G_(n+1) = frac(h, L) "and" i_n = i_(n+1) -G_(n+1)u_(n+1) $

#figure(caption: "Inductor companion model for Backwards Euler.", zap.circuit({
  import zap: *
  set-style(zap: (variant: "ieee"))

  wire((0, 0), (3, 0), i: $i_(n+1)$)
  wire((3, 0), (3, -0.5))
  wire((2, -0.5), (4, -0.5))
  node("n1", (3, -0.5))
  resistor("r1", (2, -0.5), (2, -3), label: $frac(L, h)$)
  isource("i1", (4, -0.5), (4, -3), label: $i_n$)
  wire((2, -3), (4, -3))
  wire((3, -3), (3, -3.5))
  node("n2", (3, -3))
  wire((0, -3.5), (3, -3.5))
  draw.line((1, -1.5), (1, -0.5), mark: (end: ">", fill: black))
  draw.line((1, -2), (1, -3), mark: (end: ">", fill: black))
  draw.content((1, -1.75), $u_(n+1)$)
  draw.content((1, -3.25), $-$)
  draw.content((1, -0.25), $+$)
}))

=== Mosfet

#figure(caption: "An n-channel Mosfet.", zap.circuit({
  import zap: *
  set-style(zap: (variant: "ieee"))
  nmos("id", (0, 0))
  node("above_d", (rel: (0, 1), to: "id.d"), label: (content: $d$, anchor: "west"))
  node("left_g", (rel: (-1, 0), to: "id.g"), label: $g$)
  node("below_s", (rel: (0, -1), to: "id.s"), label: (content: $s$, anchor: "west"))
  wire("above_d", "id.d", i: (content: $i_d$, anchor: "west"))
  wire("left_g", "id.g", i: (content: $i_g$, anchor: "south"))
  wire("below_s", "id.s", i: (content: $i_s$, anchor: "west"))
}))

Transconductance parameter:
$ beta = mu_n C_(o x) frac(W, L) $

Channel-length modulation parameter #sym.lambda. Can be set to zero to disable the effect. Usually in the range 0.01 - 0.05 $V^(-1)$.



$
  i_d = cases(
    0"," & "if" v_(g s) < V_(t h),
    beta [(v_(g s) - V_(t h)) v_(d s) - frac(v_(d s)^2, 2)]"," & "if" 0 <= v_(d s) <= v_(g s) - V_(t h) "(linear region)",
    frac(beta, 2) (v_(g s) - V_(t h))^2 (1 + lambda v_(d s)) "," & "if" 0 <= v_(g s) - V_(t h) <= v_(d s) "(saturation region)",
  )
$

with:
$ i_s = -i_d "and" i_g = 0 $


$
  G_(d s) = frac(partial i_d, partial v_(d s)) = cases(
    0"," & "if" v_(g s) <= v_(t h) "(cut-off)"
    , beta (v_(g s) - V_(t h) - v_(d s)) "," & "if" 0 <= v_(d s) <= v_(g s) - V_(t h) "(linear region)",
    frac(beta, 2) lambda (v_(g s) - V_(t h))^2"," & "if" 0 <= v_(g s) - V_(t h) <= v_(d s) "(saturation region)",
  ),
$

$
  g_m = frac(partial i_d, partial v_(g s)) = cases(
    0"," & "if" v_(g s) <= V_(t h) "(cut-off)",
    beta v_(d s) "," & "if" 0 <= v_(d s) <= v_(g s) - V_(t h) "(linear region)",
    beta (v_(g s) - V_(t h)) (1 + lambda v_(d s)) "," & "if" 0 <= v_(g s) - V_(t h) <= v_(d s) "(saturation region)",
  ),
$

$ I_(e q) = i_d - G_(d s) v_(d s) - g_m v_(g s) $



#figure(
  table(
    columns: 5,
    align: horizon,
    stroke: none,
    table.header(
      table.hline(),
      [], $d$, $s$, $g$, "RHS",
      table.hline(),
      $d$, $+G_(d s)$, $-(G_(d s)+g_m)$, $g_m$, $-I_"eq"$,
      $s$, $-G_(d s)$, $+(G_(d s)+g_m)$, $g_m$, $+I_"eq"$,
      $g$, $0$, $0$, $0$, $+0$,
      table.hline(),
    ),
  ),
  caption: [Element stamps for a Mosfet in group 1.],
)





=== Voltage Source

In the conductance matrix the stamps for a voltage source are given by:

If the positive terminal is connect to node `i` and the node is not grounded, the stamp is: 1


== Analyses


$
  abs(x_"new" - x_"old") <= "relative_tolerance" * max(abs(x_"new"), abs(x_"old")) +
  cases(
    "current_absolute_tolerance",
    "voltage_absolute_tolerance",
  )
$



=== DC

During DC analysis, the circuit is analyzed under steady-state conditions with all capacitors treated as open circuits and all inductors treated as short circuits.




==== Diode IV Curve

#let v1 = 1
#let r1 = 1000
$cases(V_1 = 1, R_1 = 1000)$

#figure(caption: "Diode IV Curve", zap.circuit({
  import zap: *
  set-style(zap: (variant: "ieee"))
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


