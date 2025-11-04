#[cfg(test)]
mod tests {
    use krets_parser::{elements::Element, parser::parse_circuit_description, prelude::*};

    #[test]
    fn test_parse_empty_netlist() {
        let netlist = "";
        let result = parse_circuit_description(netlist);
        assert!(matches!(result, Err(Error::EmptyNetlist)));
    }

    #[test]
    fn test_parse_netlist() {
        let netlist = "V1 5 0 2
V2 3 2 0.2
V3 7 6 2
I1 4 8 1e-3
I2 0 6 1e-3
R1 1 5 1.5
R2 1 2 1
R3 5 2 50
R4 5 6 0.1
R5 2 6 1.5
R6 3 4 0.1
R7 8 0 1e3
R8 4 0 10";
        let circuit = parse_circuit_description(netlist);

        if let Err(e) = &circuit {
            println!("Parsing failed with error: {:?}", e);
        }
        assert!(circuit.is_ok());

        let circuit = circuit.unwrap();
        assert_eq!(circuit.elements.len(), 13);
        assert_eq!(circuit.nodes.len(), 9);
    }

    #[test]
    fn test_parse_rectifier() {
        let netlist = "
V1 in_ac1 0 SIN(0 5 60 0 0 0)
V2 in_ac2 0 SIN(0 -5 60 0 0 0)

D1 in_ac1 out_dc DMOD
D2 in_ac2 out_dc DMOD  
D3 0 in_ac1 DMOD
D4 0 in_ac2 DMOD

C1 out_dc 0 100u
R1 out_dc 0 1k 

.model DMOD D (is=1e-9)


.control
  tran 0.1ms 50ms
  run
  plot v(in_ac1,in_ac2) v(out_dc)
.endc
.end";
        let circuit = parse_circuit_description(netlist);

        for element in circuit.as_ref().unwrap().elements.iter() {
            if let Element::Diode(diode) = element {
                assert_eq!(diode.model_name, "DMOD");
                assert_eq!(diode.model.saturation_current, 1e-9);
            }
        }

        if let Err(e) = &circuit {
            println!("Parsing failed with error: {:?}", e);
        }
        assert!(circuit.is_ok());
    }

    #[test]
    fn test_with_subckt() {
        let netlist = "
xdiv1 10 7 0 vdivide
.subckt vdivide 1 2 3
r1 1 2 10K
r2 2 3 5K
.ends
";
        let circuit = parse_circuit_description(netlist);

        if let Err(e) = &circuit {
            println!("Parsing failed with error: {:?}", e);
        }
        assert!(circuit.is_ok());

        let circuit = circuit.unwrap();
        assert_eq!(circuit.subcircuit_definitions.len(), 1); // One subcircuit definition
    }
}
