/// A macro for generating arbitrary documented code items
macro_rules! doc_comment {
    ($docs:expr, $($item:tt)*) => {
        #[doc = $docs]
        $($item)*
    };
}

/*********** various macros for generating documentation for duplicated functions *****************/

macro_rules! doc_tag_bits {
    () => {
        "The number of available tag bits for this type."
    };
}

macro_rules! doc_tag_mask {
    () => {
        "The bitmask for the lower bits available for storing the tag value."
    };
}

macro_rules! doc_ptr_mask {
    () => {
        "The bitmask for the (higher) bits for storing the pointer itself."
    };
}

macro_rules! doc_null {
    ($example_type_path:path) => {
        concat!(
            "Creates a new `null` pointer.\n\n\
            # Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let ptr = MarkedPtr::null();\n\
            assert_eq!(ptr.decompose(), (ptr::null_mut(), 0));\n```"
        )
    };
}

macro_rules! doc_new {
    ($example_type_path:path) => {
        concat!(
            "Creates a new unmarked pointer.\n\n\
            # Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::new(reference);\n\
            assert_eq!(ptr.decompose(), (reference as *mut _, 0));\n```"
        )
    };
}

macro_rules! doc_cast {
    () => {
        "Casts to a pointer of another type."
    };
}

macro_rules! doc_compose {
    () => {
        "Composes a new marked pointer from a raw `ptr` and a `tag` value.\
    
        The supplied `ptr` is assumed to be well-aligned (i.e. has no tag bits set) and calling \
        this function may lead to unexpected results when this is not the case."
    };
}

macro_rules! doc_is_null {
    () => {
        "Returns `true` if the marked pointer is `null`."
    };
}

macro_rules! doc_clear_tag {
    ($example_type_path:path) => {
        concat!(
            "Clears the marked pointer's tag value.\n\n\
            # Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::compose(reference, 0b11);\n\
            assert_eq!(ptr.clear_tag(), MarkedPtr::new(reference));\n```"
        )
    };
}

macro_rules! doc_split_tag {
    ($example_type_path:path) => {
        concat!(
            "Splits the tag value from the marked pointer, returning both the cleared pointer and the \
            separated tag value.\n\n\
            # Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::compose(reference, 0b11);\n\
            assert_eq!(ptr.split_tag(), (MarkedPtr::new(reference), 0b11));\n```"
        )
    };
}

macro_rules! doc_set_tag {
    ($example_type_path:path) => {
        concat!(
            "Sets the marked pointer's tag value to `tag` and overwrites any previous value.\n\n\
            # Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::compose(reference, 0b11);\n\
            assert_eq!(ptr.set_tag(0b10).decompose(), (reference as *mut _, 0b10));\n```"
        )
    };
}

macro_rules! doc_update_tag {
    () => {
        "Updates the marked pointer's tag value to the result of `func`, which is called with the \
        current tag value."
    };
    ($example_type_path:path) => {
        concat!(
            "Updates the marked pointer's tag value to the result of `func`, which is called with \
            the current tag value.\n\n\
            # Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;
            let ptr = MarkedPtr::compose(reference, 0b11);\n\
            let ptr = ptr.update_tag(|tag| tag - 1);\n\
            assert_eq!(ptr.decompose(), (reference as *mut _, 0b10));\n```"
        )
    };
}

macro_rules! doc_add_tag {
    () => {
        "Adds `value` to the current tag *without* regard for the previous value.\
        \
        This method does not perform any checks so it may silently overflow the tag bits, result \
        in a pointer to a different value, a null pointer or an unaligned pointer."
    };
}

macro_rules! doc_sub_tag {
    () => {
        "Subtracts `value` from the current tag *without* regard for the previous value.\
        \
        This method does not perform any checks so it may silently overflow the tag bits, result \
        in a pointer to a different value, a null pointer or an unaligned pointer."
    };
}

macro_rules! doc_as_ref {
    () => {
        "Decomposes the marked pointer, returning an optional reference and discarding the tag.\n\n\
        # Safety\n\n\
        While this method and its mutable counterpart are useful for null-safety, it is \
        important to note that this is still an unsafe operation because the returned value \
        could be pointing to invalid memory.\n\n\
        Additionally, the lifetime 'a returned is arbitrarily chosen and does not necessarily \
        reflect the actual lifetime of the data."
    };
}

macro_rules! doc_as_mut {
    ($ty_ident:ident) => {
        concat!(
            "Decomposes the marked pointer, returning an optional *mutable* reference and discarding \
            the tag.\n\n\
            # Safety\n\n\
            The same safety caveats as with [`as_ref`][", stringify!($ty_ident),
            "::as_ref] apply."
        )
    }
}

macro_rules! doc_decompose_ref {
    ($ty_ident:ident) => {
        concat!(
            "Decomposes the marked pointer, returning an optional reference and the separated tag.\n\n\
            # Safety\n\n\
            The same safety caveats as with [`as_ref`][", stringify!($ty_ident),
            "::as_ref] apply."
        )
    };
}

macro_rules! doc_decompose_mut {
    ($ty_ident:ident) => {
        concat!(
            "Decomposes the marked pointer, returning an optional *mutable* reference and the \
            separated tag.\n\n\
            # Safety\n\n\
            The same safety caveats as with [`as_ref`][",
            stringify!($ty_ident),
            "::as_ref] apply."
        )
    };
}

macro_rules! doc_decompose {
    () => {
        "Decomposes the marked pointer, returning the raw pointer and the tag value."
    };
}

macro_rules! doc_decompose_ptr {
    () => {
        "Decomposes the marked ptr, returning only the separated raw pointer."
    };
}

macro_rules! doc_decompose_tag {
    () => {
        "Decomposes the marked ptr, returning only the separated tag value."
    };
}
