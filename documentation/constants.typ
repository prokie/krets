#let boltzmann_constant = 1.380649e-23
#let elementary_charge = 1.602176634e-19
#let standard_temperature = 300
#let thermal_voltage = boltzmann_constant * standard_temperature / elementary_charge
#let reverse_saturation_current = 1e-12


#let shockley_diode_current(diode_voltage) = (
  reverse_saturation_current * (calc.exp(diode_voltage / thermal_voltage) - 1)
)

#let shockley_diode_conductance(diode_voltage) = (
  (reverse_saturation_current / thermal_voltage) * calc.exp(diode_voltage / thermal_voltage)
)
