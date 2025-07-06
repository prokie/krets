#import "constants.typ": reverse_saturation_current, thermal_voltage
#import "@preview/zap:0.2.0"
#import "@preview/cetz:0.4.0"
#import "@preview/cetz-plot:0.1.2"

== Constants


The following physical constants are used throughout this document:


$k_B = 1.380649 dot 10^(-23)$ (Boltzmann constant)

$q = 1.602176634 dot 10^(-19)$ (Elementary charge)


$T = 300$ (Standard temperature)



$V_T = frac(k_B T, q) approx #calc.round(thermal_voltage, digits: 5)$ (Thermal voltage at 300K)


$I_S = 1 dot 10^-12$ (reverse saturation current)


#reverse_saturation_current


