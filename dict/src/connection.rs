use crate::{builder::matrix_def::MatrixDef, dict::DictReadWrite};

// ConnectionTable represents a connection matrix of morphs.
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct ConnectionTable {
    row: usize,
    col: usize,
    data: Vec<i16>,
}

impl ConnectionTable {
    pub fn get(&self, row: usize, col: usize) -> i16 {
        self.data[self.row * col + row]
    }
}

impl From<MatrixDef> for ConnectionTable {
    fn from(m: MatrixDef) -> Self {
        ConnectionTable {
            row: m.row,
            col: m.col,
            data: m.data,
        }
    }
}

impl DictReadWrite for ConnectionTable {
    fn from_dict<R: std::io::Read>(r: &mut R) -> std::io::Result<Self> {
        let mut buf = [0; 8];
        r.read_exact(&mut buf)?;
        let row = usize::from_le_bytes(buf);
        r.read_exact(&mut buf)?;
        let col = usize::from_le_bytes(buf);
        let mut data = vec![0; row * col];
        let mut buf: [u8; 2] = [0; 2];
        for i in 0..row * col {
            r.read_exact(&mut buf)?;
            data[i] = i16::from_le_bytes(buf);
        }

        Ok(ConnectionTable { row, col, data })
    }

    fn write_dict<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.row.to_le_bytes())?;
        w.write_all(&self.col.to_le_bytes())?;
        for d in &self.data {
            w.write_all(&d.to_le_bytes())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get() {
        let m = MatrixDef {
            row: 2,
            col: 2,
            data: vec![0, 1, 2, 3],
        };
        let ct = ConnectionTable::from(m.clone());
        for i in 0..ct.row {
            for j in 0..ct.col {
                let expected = (j * m.row + i) as i16;
                assert_eq!(ct.get(i, j), expected);
            }
        }
    }

    #[test]
    fn test_write_read_dict() {
        let m = MatrixDef {
            row: 2,
            col: 2,
            data: vec![0, 1, 2, 3],
        };
        let ct = ConnectionTable::from(m.clone());
        let mut buf = Vec::new();
        ct.write_dict(&mut buf).unwrap();
        let mut cursor = std::io::Cursor::new(&buf);
        let ct2 = ConnectionTable::from_dict(&mut cursor).unwrap();
        assert_eq!(ct, ct2);
    }
}
