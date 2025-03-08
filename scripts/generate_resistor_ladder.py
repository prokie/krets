from pathlib import Path


def generate_resistor_ladder(n: int) -> str:
    """
    Generates an N-sized SPICE circuit resistor ladder.
    
    Args:
        n: The number of resistors in the ladder.
    
    Returns:
        The SPICE circuit description.
    """
    if n < 1:
        raise ValueError("The number of resistors must be at least 1.")
    
    circuit = ["% N-sized resistor ladder"]
    circuit.append("V1 1 0 1")  
    
    for i in range(1, n + 1):
        if i == n:
            circuit.append(f"R{i} {i} 0 1000") 
        else:
            circuit.append(f"R{i} {i} {i+1} 1000")  
    
    
    return "\n".join(circuit)

if __name__ == "__main__":
    n = 500  # Example value, you can change this to any number of resistors you need
    circuit_description = generate_resistor_ladder(n)
    script_dir = Path(__file__).parent
    output_dir = script_dir/ Path(f"../circuits/resistor_ladder_{n}")
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f"resistor_ladder_{n}.cir"
    output_file.write_text(circuit_description)