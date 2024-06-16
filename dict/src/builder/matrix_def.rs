use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

// MatrixDef represents matrix.def
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct MatrixDef {
    pub row: usize,
    pub col: usize,
    pub data: Vec<i16>,
}

pub fn parse_matrix_def(path: &Path) -> Result<MatrixDef, anyhow::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    parse(&mut reader)
}

fn parse(reader: &mut BufReader<File>) -> Result<MatrixDef, anyhow::Error> {
    let mut lines = reader.lines();
    // row col
    let line = lines.next().expect("Failed to read row and col")?;
    let values = line
        .split_whitespace()
        .map(|value| value.parse::<usize>().expect("Failed to parse row and col"))
        .collect::<Vec<usize>>();
    if values.len() != 2 {
        return Err(anyhow::anyhow!("Invalid row and col: {:?}", line));
    }
    let row = values[0];
    let col = values[1];
    let mut data = vec![0; row * col];
    for line in lines.map(|line| line.expect("Failed to read matrix value")) {
        let values = line
            .split_whitespace()
            .map(|value| value.parse::<i64>().expect("Failed to parse matrix value"))
            .collect::<Vec<i64>>();
        if values.len() != 3 {
            return Err(anyhow::anyhow!("Invalid matrix value: {:?}", line));
        }
        // i64 -> usize return err
        let r = usize::try_from(values[0])?;
        let c = usize::try_from(values[1])?;
        let value = i16::try_from(values[2])?;
        if r >= row || c >= col {
            return Err(anyhow::anyhow!("Invalid matrix index: {:?}", line));
        }
        data[c * row + r] = value;
    }
    Ok(MatrixDef { row, col, data })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        // Create a temporary file
        let path = std::env::temp_dir().join("matrix.def");
        std::fs::write(&path, "2 2\n0 0 1\n0 1 2\n1 0 3\n1 1 4\n").unwrap();
        let matrix_def = parse_matrix_def(&path);
        assert_eq!(
            matrix_def.unwrap(),
            MatrixDef {
                row: 2,
                col: 2,
                data: vec![1, 3, 2, 4]
            }
        );
        // Remove the temporary file
        std::fs::remove_file(&path).unwrap();
    }
}
