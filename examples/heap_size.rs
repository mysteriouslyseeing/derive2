use derive2::derive2;

trait HeapSize {
    fn heap_size(&self) -> usize;
}

impl HeapSize for i32 {
    fn heap_size(&self) -> usize {
        0
    }
}
impl HeapSize for u8 {
    fn heap_size(&self) -> usize {
        0
    }
}
impl HeapSize for String {
    fn heap_size(&self) -> usize {
        self.capacity()
    }
}
impl<T> HeapSize for Vec<T> {
    fn heap_size(&self) -> usize {
        self.capacity() * size_of::<T>()
    }
}

macro_rules! heap_size {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_ty:ty
            ),*
            $(,)?
        }
    ) => {
        #[automatically_derived]
        impl HeapSize for $name {
            fn heap_size(&self) -> usize {
                0 $(+ HeapSize::heap_size(&self.$field_name))*
            }
        }
    };
}

#[derive2(heap_size)]
#[derive(Debug, Default)]
pub struct LotsOfFields {
    a: i32,
    b: u8,
    c: String,
    d: Vec<i32>,
}

fn main() {
    let mut lof = LotsOfFields::default();
    println!("lof = {lof:?}, heap_size is {}", lof.heap_size());

    assert!(lof.heap_size() == 0);

    lof.a = 42;
    lof.b = u8::MAX;
    println!("lof = {lof:?}, heap_size is {}", lof.heap_size());

    assert!(lof.heap_size() == 0);

    lof.c.push_str("hello world");
    println!("lof = {lof:?}, heap_size is {}", lof.heap_size());

    assert!(lof.heap_size() >= "hello_world".len());

    lof.d = vec![1, 2, 3];
    println!("lof = {lof:?}, heap_size is {}", lof.heap_size());

    assert!(lof.heap_size() >= size_of::<[i32; 3]>() + "hello_world".len());

    lof.c = String::new();
    println!("lof = {lof:?}, heap_size is {}", lof.heap_size());

    assert!(lof.heap_size() >= size_of::<[i32; 3]>());

    lof.d = Vec::new();

    assert!(lof.heap_size() == 0);

    println!("All assertions passed!");
}
