/// Collection of macros for generating documentation.

/// A macro for generating arbitrary documented code items
macro_rules! doc_comment {
    ($docs:expr, $($item:tt)*) => {
        #[doc = $docs]
        $($item)*
    };
}

/********** various macros for generating documentation for constants *****************************/

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

/********** macros for generating marked pointers *************************************************/

macro_rules! doc_null {
    ($example_type_path:path) => {
        concat!(
            doc_null!(),
            "\n\n# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let ptr = MarkedPtr::null();\n\
            assert_eq!(ptr.decompose(), (ptr::null_mut(), 0));\n```"
        )
    };
    () => {
        "Creates a new `null` pointer."
    };
}

macro_rules! doc_from_usize {
    ("nullable", $example_type_path:path) => {
        concat!(
            doc_from_usize!(),
            "\n\n# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let ptr = MarkedPtr::from_usize(0b11);\n\
            assert_eq!(ptr.decompose(), (ptr::null_mut(), 0b11));\n```"
        )
    };
    () => {
        "Creates a new pointer from the numeric (integer) representation of a potentially marked pointer."
    };
}

macro_rules! doc_into_raw {
    () => {
        "Returns the internal representation of the pointer *as is*, i.e. any potential tag value is **not** stripped."
    };
}

macro_rules! doc_into_usize {
    () => {
        "Returns the numeric (integer) representation of the pointer with its tag value."
    };
}

macro_rules! doc_dangling {
    () => {
        "Creates a new pointer that is dangling but well aligned."
    };
}

macro_rules! doc_new {
    ($example_type_path:path) => {
        concat!(
            doc_new!(),
            "\n\n# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::new(reference);\n\
            assert_eq!(ptr.decompose(), (reference as *mut _, 0));\n```"
        )
    };
    () => {
        "Creates a new unmarked pointer."
    };
}

macro_rules! doc_cast {
    () => {
        "Casts to a pointer of another type."
    };
}

macro_rules! doc_compose {
    () => {
        "Composes a new marked pointer from a raw `ptr` and a `tag` value.\n\n\
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
    ("non-null" $example_type_path:path) => {
        concat!(
            doc_clear_tag!(),
            "# Examples\n\n\
            ```\nuse core::ptr::NonNull;\n\n\
            type MarkedNonNull = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedNonNull::compose(NonNull::from(reference), 0b11);\n\
            assert_eq!(ptr.clear_tag(), MarkedNonNull::from(reference));\n```"
        )
    };
    ($example_type_path:path) => {
        concat!(
            doc_clear_tag!(),
            "# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::compose(reference, 0b11);\n\
            assert_eq!(ptr.clear_tag(), MarkedPtr::new(reference));\n```"
        )
    };
    () => {
        "Clears the marked pointer's tag value.\n\n"
    };
}

macro_rules! doc_split_tag {
    ("non-null" $example_type_path:path) => {
        concat!(
            doc_split_tag!(),
            "# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedNonNull = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedNonNull::compose(NonNull::from(reference), 0b11);\n\
            assert_eq!(ptr.split_tag(), (MarkedNonNull::from(reference), 0b11));\n```"
        )
    };
    ($example_type_path:path) => {
        concat!(
            doc_split_tag!(),
            "# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::compose(reference, 0b11);\n\
            assert_eq!(ptr.split_tag(), (MarkedPtr::new(reference), 0b11));\n```"
        )
    };
    () => {
        "Splits the tag value from the marked pointer, returning both the cleared pointer and the \
        separated tag value.\n\n"
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
        "Adds `value` to the current tag *without* regard for the previous value.\n\n\
        This method does not perform any checks so it may silently overflow the tag bits, result \
        in a pointer to a different value, a null pointer or an unaligned pointer."
    };
}

macro_rules! doc_sub_tag {
    () => {
        "Subtracts `value` from the current tag *without* regard for the previous value.\n\n\
        This method does not perform any checks so it may silently overflow the tag bits, result \
        in a pointer to a different value, a null pointer or an unaligned pointer."
    };
}

macro_rules! doc_as_ref {
    ("bounded") => {
        concat!(doc_as_ref!(), "\n\n# Safety\n\n", doc_as_ref!("safety"))
    };
    ("unbounded") => {
        concat!(
            doc_as_ref!("bounded"),
            "\n\nAdditionally, the lifetime 'a returned is arbitrarily chosen and does not \
            necessarily reflect the actual lifetime of the data."
        )
    };
    ("safety") => {
        "While this method and its mutable counterpart are useful for null-safety, it is \
        important to note that this is still an unsafe operation because the returned value \
        could be pointing to invalid memory."
    };
    () => {
        "Decomposes the marked pointer, returning an optional reference and discarding the tag."
    };
}

macro_rules! doc_as_mut {
    ($self_ident:ident) => {
        concat!(
            "Decomposes the marked pointer, returning an optional *mutable* reference and discarding \
            the tag.\n\n\
            # Safety\n\n\
            The same safety caveats as with [`as_ref`][", stringify!($self_ident),
            "::as_ref] apply."
        )
    }
}

macro_rules! doc_decompose_ref {
    ($self_ident:ident) => {
        concat!(
            "Decomposes the marked pointer, returning a reference and the separated tag.\n\n\
            # Safety\n\n\
            The same safety caveats as with [`as_ref`][",
            stringify!($self_ident),
            "::as_ref] apply."
        )
    };
}

macro_rules! doc_decompose_mut {
    ($self_ident:ident) => {
        concat!(
            "Decomposes the marked pointer, returning a *mutable* reference and the \
            separated tag.\n\n\
            # Safety\n\n\
            The same safety caveats as with [`as_ref`][",
            stringify!($self_ident),
            "::as_ref] apply."
        )
    };
}

macro_rules! doc_decompose {
    () => {
        "Decomposes the marked pointer, returning the raw pointer and the separated tag value."
    };
}

macro_rules! doc_decompose_ptr {
    () => {
        "Decomposes the marked pointer, returning only the separated raw pointer."
    };
}

macro_rules! doc_decompose_non_null {
    () => {
        "Decomposes the marked pointer, returning only the separated raw [`NonNull`] pointer."
    };
}

macro_rules! doc_decompose_tag {
    () => {
        "Decomposes the marked pointer, returning only the separated tag value."
    };
}

/********** macros for generating atomic marked pointers *******************************************/

macro_rules! doc_load {
    () => {
        "Loads the value of the atomic marked pointer.\n\n\
        `load` takes an [`Ordering`] argument which describes the memory ordering of this \
        operation.\n\
        Possible values are [`SeqCst`][seq_cst], [`Acquire`][acq] and [`Relaxed`][rlx].\n\n\
        # Panics\n\n\
        Panics if `order` is [`Release`][rel] or [`AcqRel`][acq_rel].\n\n\
        [rlx]: Ordering::Relaxed\n\
        [acq]: Ordering::Acquire\n\
        [rel]: Ordering::Release\n\
        [acq_rel]: Ordering::AcqRel\n\
        [seq_cst]: Ordering::SeqCst"
    };
}
