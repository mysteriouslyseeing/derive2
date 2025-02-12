use derive2::derive2;

trait HeapSize {
    fn heap_size(&self) -> usize;
}

impl HeapSize for String {
    fn heap_size(&self) -> usize {
        self.capacity()
    }
}

macro_rules! heap_size {
    // @impl implements the trait for the given struct
    (
        @impl
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$($field_meta_tokens:tt)*])*
                $field_vis:vis $field_name:ident: $field_ty:ty
            ),*
            $(,)?
        }
    ) => {
        #[automatically_derived]
        impl $crate::HeapSize for $name {
            fn heap_size(&self) -> usize {
                0 $(+ $crate::heap_size!(
                    @parse_attrs &self.$field_name, $(#[$($field_meta_tokens)*])*
                ))*
            }
        }
    };
    // @parse_attrs either calls HeapSize::heap_size on $else, returns 0, or calls $with_expr on
    // $else, depending on what attributes are found

    // This will match heap_size(skip), just using 0
    (
        @parse_attrs
        $else:expr,
        #[heap_size(skip)]
        $(#[$($rest:tt)*])*
    ) => {
        0
    };
    // This will match heap_size(with = <expression>), calling (expression)($else)
    (
        @parse_attrs
        $else:expr,
        #[heap_size(with = $with_expr:expr)]
        $(#[$($rest:tt)*])*
    ) => {
        ($with_expr)($else)
    };
    // This will match an unrecognised #[heap_size] attribute, raising a compile error
    (
        @parse_attrs
        $else:expr,
        #[heap_size $($other:tt)*]
        $(#[$($rest:tt)*])*
    ) => {
        compile_error!(concat!(
            "Unrecognised heap_size attribute: ",
            stringify!(#[heap_size $($other)*])
        ))
    };
    // This will match a non-#[heap_size] attribute, skipping it
    (
        @parse_attrs
        $else:expr,
        #[$($first:tt)*]
        $(#[$($rest:tt)*])*
    ) => {
        {
            $crate::heap_size! {
                @parse_attrs
                $else,
                $(#[$($rest)*])*
            }
        }
    };
    // This will match the case where no heap_size attributes were found, using HeapSize::heap_size
    // instead.
    (
        @parse_attrs
        $else:expr,
    ) => {
        $crate::HeapSize::heap_size($else)
    };
    // Because all macro input that does not start with @impl will get forwarded to @impl in the
    // final pattern, this means the macro input was not recognised. This will happen if
    // #[derive2(heap_size)] was used on an enum, or a struct with generics, for example.
    (@impl $($other:tt)*) => {
        compile_error!{
            concat!("unrecognised macro input: ", stringify!($($other)*))
        }
    };
    // Forward un-attributed macro input to @impl
    ($($other:tt)*) => {
        $crate::heap_size! {
            @impl $($other)*
        }
    }
}
use heap_size;

#[derive2(heap_size)]
#[derive(Debug, Default)]
struct LotsOfFields {
    #[heap_size(skip)]
    a: i32,
    #[heap_size(skip)]
    b: u8,
    c: String,
    #[heap_size(with = |v: &Vec<i32>| { v.capacity() * size_of::<i32>() })]
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
