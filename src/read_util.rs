#![macro_use]

#[macro_export]
macro_rules! read_int {
    ($int_type:ty, from $src:expr, with $dest:expr) => {{
        const PRIM_SIZE: usize = std::mem::size_of::<$int_type>();
        $src.read_exact(&mut $dest[0..PRIM_SIZE]).unwrap();
        <$int_type>::from_be_bytes((&$dest[0..PRIM_SIZE]).try_into().unwrap())
    }};
    ($int_type:ty, from $src:expr) => {{
        const PRIM_SIZE: usize = std::mem::size_of::<$int_type>();
        let mut dest = [0u8; PRIM_SIZE];
        $src.read_exact(&mut dest[0..PRIM_SIZE]).unwrap();
        <$int_type>::from_be_bytes(dest)
    }};
}
