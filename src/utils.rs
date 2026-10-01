pub fn dir(path: &str) -> String {
    let dir = &path[..path.rfind('/').unwrap()];
    if dir.is_empty() { "/" } else { dir }.to_owned()
}

#[macro_export]
macro_rules! arc {
    ($id:ident => $inner_id:ident { $($field:ident: $type:ty),* $(,)? }) => {
        arc!($id => $inner_id { $($field: $type),* }, pub new);
    };
    ($id:ident => $inner_id:ident { $($field:ident: $type:ty),* $(,)? }, $new_vis:vis new) => {
        arc!($id => $inner_id { $($field: $type),* }, without new);

        impl $id {
            $new_vis fn new($($field: $type),*) -> Self {
                Self(std::sync::Arc::new($inner_id { $($field),* }))
            }
        }
    };
    ($id:ident => $inner_id:ident { $($field:ident: $type:ty),* $(,)? } with FromRef) => {
        arc!($id => $inner_id { $($field: $type),* }, without new);

        $(
            impl axum::extract::FromRef<$id> for $type {
                fn from_ref(input: &$id) -> Self {
                    input.$field.clone()
                }
            }
        )*
    };
    ($id:ident => $inner_id:ident { $($field:ident: $type:ty),* $(,)? }, without new) => {
        #[derive(Clone)]
        pub struct $id(std::sync::Arc<$inner_id>);

        pub struct $inner_id { $($field: $type),* }

        impl std::ops::Deref for $id {
            type Target = $inner_id;

            fn deref(&self) -> &Self::Target { &self.0 }
        }
    }
}
