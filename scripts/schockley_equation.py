import math



# Boltzmann constant in J/K
KB = 1.380649e-23

# Elementary charge in C
Q = 1.602176634e-19

# Standard temperature in Kelvin
TEMPERATURE = 300.0

# Thermal voltage at 300K in V
THERMAL_VOLTAGE = KB * TEMPERATURE / Q


SATURATION_CURRENT = 1e-12 

PARASITIC_RESISTANCE = 0

EMISSION_COEFFICIENT = 1.0

DIODE_VOLTAGE = 0


conductance = (SATURATION_CURRENT / THERMAL_VOLTAGE) * math.exp(DIODE_VOLTAGE / THERMAL_VOLTAGE)
 
print(conductance)