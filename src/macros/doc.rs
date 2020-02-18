/// All macros for generating documentation.

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
    ("non-null" $example_type_path:path) => {
        concat!(
            doc_set_tag!(),
            "\n\n# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedNonNull = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedNonNull::compose(NonNull::from(reference), 0b11);\n\
            assert_eq!(ptr.set_tag(0b10).decompose(), (NonNull::from(reference), 0b10));\n```"
        )
    };
    ($example_type_path:path) => {
        concat!(
            doc_set_tag!(),
            "\n\n# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedPtr = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedPtr::compose(reference, 0b11);\n\
            assert_eq!(ptr.set_tag(0b10).decompose(), (reference as *mut _, 0b10));\n```"
        )
    };
    () => {
        "Sets the marked pointer's tag value to `tag` and overwrites any previous value."
    };
}

macro_rules! doc_update_tag {
    ("non-null" $example_type_path:path) => {
        concat!(
            doc_update_tag!(),
            "\n\n# Examples\n\n\
            ```\nuse core::ptr;\n\n\
            type MarkedNonNull = ",
            stringify!($example_type_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = MarkedNonNull::compose(reference, 0b11);\n\
            assert_eq!(ptr.update_tag(|tag| tag - 2).decompose(), (NonNull::from(reference), 0b01));\n```"
        )
    };
    ($example_type_path:path) => {
        concat!(
            doc_update_tag!(),
            "\n\n# Examples\n\n\
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
    () => {
        "Updates the marked pointer's tag value to the result of `func`, which is called with the \
        current tag value."
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

macro_rules! doc_store {
    () => {
        "Stores a value into the atomic marked pointer.\n\n\
        `store` takes an [`Ordering`] argument which describes the memory ordering of this operation.\n\
        Possible values are [`SeqCst`][seq_cst], [`Release`][rel] and [`Relaxed`][rlx].\n\n\
        # Panics\n\n\
        Panics if `order` is [`Acquire`][acq] or [`AcqRel`][acq_rel].\n\n\
        [rlx]: Ordering::Relaxed\n\
        [acq]: Ordering::Acquire\n\
        [rel]: Ordering::Release\n\
        [acq_rel]: Ordering::AcqRel\n\
        [seq_cst]: Ordering::SeqCst"
    };
}

macro_rules! doc_swap {
    () => {
        "Stores a value into the atomic marked pointer and returns the previous value.\n\n\
        `swap` takes an [`Ordering`] argument which describes the memory ordering of this \
        operation. All ordering modes are possible. Note that using [`Acquire`][acq] makes the \
        store part of this operation [`Relaxed`][rlx], and using [`Release`][rel] makes the load \
        part [`Relaxed`][rlx].\n\n\
        [rlx]: Ordering::Relaxed\n\
        [acq]: Ordering::Acquire\n\
        [rel]: Ordering::Release\n"
    };
}

macro_rules! doc_compare_and_swap {
    () => {
        "Stores a value into the pointer if the current value is the same as `current`.\n\n\
        The return value is always the previous value. If it is equal to `current`, then the value \
        was updated.\n\n\
        `compare_and_swap` also takes an [`Ordering`] argument which describes the memory ordering \
        of this operation. Notice that even when using [`AcqRel`][acq_rel], the operation might \
        fail and hence just perform an `Acquire` load, but not have `Release` semantics. Using \
        [`Acquire`][acq] makes the store part of this operation [`Relaxed`][rlx] if it happens, \
        and using [`Release`][rel] makes the load part [`Relaxed`][rlx].\n\n\
        [rlx]: Ordering::Relaxed\n\
        [acq]: Ordering::Acquire\n\
        [rel]: Ordering::Release\n\
        [acq_rel]: Ordering::AcqRel\n\
        [seq_cst]: Ordering::SeqCst"
    };
}

macro_rules! doc_compare_exchange {
    ("strong", $fn_ident:expr) => {
        concat!(
            doc_compare_exchange!(),
            "\n\n",
            doc_compare_exchange!("ordering", $fn_ident),
        )
    };
    ("weak", $fn_ident:expr) => {
        concat!(
            doc_compare_exchange!(),
            "\n\n\
            Unlinke `compare_exchange`, this function is allowed to spuriously fail even when the \
            comparison succeeds, which can result in more efficient code on some platforms. The \
            return value is a result indicating whether the new value was written and containing \
            the previous value.\n\n",
            doc_compare_exchange!("ordering", $fn_ident),
        )
    };
    ("ordering", $fn_ident:expr) => {
        concat!(
            $fn_ident,
            " takes takes two [`Ordering`] arguments to describe the memory ordering of this \
            operation. The first describes the required ordering if the operation succeeds while \
            the second describes the required ordering when the operation fails. Using \
            [`Acquire`][acq] as success ordering makes store part of this operation \
            [`Relaxed`][rlx], and using [`Release`][rel] makes the successful load \
            [`Relaxed`][rlx]. The failure ordering can only be [`SeqCst`][seq_cst], \
            [`Acquire`][acq] or [`Relaxed`][rlx] and must be equivalent or weaker than the success \
            ordering.\n\n\
            [rlx]: Ordering::Relaxed\n\
            [acq]: Ordering::Acquire\n\
            [rel]: Ordering::Release\n\
            [seq_cst]: Ordering::SeqCst"
        )
    };
    () => {
        "Stores a value into the pointer if the current value is the same as `current`.\n\n\
        The return value is a result indicating whether the new value was written and containing \
        the previous value. On success this value is guaranteed to be equal to `current`."
    }
}

macro_rules! doc_fetch_and_x {
    ("note") => {
        "This operation directly and unconditionally alters the internal numeric representation \
        of the atomic marked pointer. Hence there is no way to reliably guarantee the operation \
        only affects the tag bits and does not overflow into the pointer bits."
    };
    ("ordering", $fn_ident:expr) => {
        concat!(
            $fn_ident,
            " takes an [`Ordering`] argument which describes the memory ordering of this operation.\n\
            All ordering modes are possible. Note that using [`Acquire`][acq] makes the store part \
            of this operation [`Relaxed`][rlx] and using [`Release`][rel] makes the load part \
            [`Relaxed`][rlx].\n\n\
            [rlx]: Ordering::Relaxed\n\
            [acq]: Ordering::Acquire\n\
            [rel]: Ordering::Release"
        )
    };
}

macro_rules! doc_fetch_add {
    ($fn_ident:expr, $example_atomic_path:path) => {
        concat!(
            doc_fetch_add!(),
            "\n\n",
            doc_fetch_and_x!("note"),
            "\n\n",
            doc_fetch_and_x!("ordering", $fn_ident),
            "\n\n# Examples\n\n\
            ```\nuse core::ptr;\n\
            use core::sync::atomic::Ordering;\n\n\
            type AtomicMarkedPtr = ",
            stringify!($example_atomic_path),
            ";\n\n\
            let reference = &mut 1;\n\
            let ptr = AtomicMarkedPtr::from(reference);\n\
            assert_eq!(\n\
                \tptr.fetch_add(1, Ordering::Relaxed).decompose(),\n\
                \t(reference as *mut _, 0b01)\n\
            );\n```"
        )
    };
    () => {
        "Adds `value` to the current tag value, returning the previous marked pointer."
    };
}

macro_rules! doc_fetch_sub {
    ($fn_ident:expr, $example_atomic_path:path, $example_ptr_path:path) => {
        concat!(
            doc_fetch_sub!(),
            "\n\n",
            doc_fetch_and_x!("note"),
            "\n\n",
            doc_fetch_and_x!("ordering", $fn_ident),
            "\n\n# Examples\n\n\
           ```\nuse core::ptr;\n\
           use core::sync::atomic::Ordering;\n\n\
           type AtomicMaredPtr = ",
            stringify!($example_atomic_path),
            ";\ntype MarkedPtr = ",
            stringify!($example_ptr_path),
            ";\n\n\
           let reference = &mut 1;\n\
           let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b10));\n\
           assert_eq!(\n\
               \tptr.fetch_sub(1, Ordering::Relaxed).decompose(),\n\
               \t(reference as *mut _, 0b01)\n\
           );\n```"
        )
    };
    () => {
        "Subtracts `value` from the current tag value, returning the previous marked pointer."
    };
}

macro_rules! doc_fetch_or {
    ($fn_ident:expr, $example_atomic_path:path, $example_ptr_path:path) => {
        concat!(
            doc_fetch_or!(),
            "\n\n",
            doc_fetch_and_x!("note"),
            "\n\n",
            doc_fetch_and_x!("ordering", $fn_ident),
            "\n\n# Examples\n\n\
           ```\nuse core::ptr;\n\
           use core::sync::atomic::Ordering;\n\n\
           type AtomicMaredPtr = ",
            stringify!($example_atomic_path),
            ";\ntype MarkedPtr = ",
            stringify!($example_ptr_path),
            ";\n\n\
           let reference = &mut 1;\n\
           let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b10));\n\
           assert_eq!(\n\
               \tptr.fetch_or(0b11, Ordering::Relaxed).decompose(),\n\
               \t(reference as *mut _, 0b11)\n\
           );\n```"
        )
    };
    () => {
        "Performs a bitwise \"or\" of `value` with the current tag value, returning the previous \
        marked pointer."
    };
}

macro_rules! doc_fetch_xor {
    ($fn_ident:expr, $example_atomic_path:path, $example_ptr_path:path) => {
        concat!(
            doc_fetch_xor!(),
            "\n\n",
            doc_fetch_and_x!("note"),
            "\n\n",
            doc_fetch_and_x!("ordering", $fn_ident),
            "\n\n# Examples\n\n\
           ```\nuse core::ptr;\n\
           use core::sync::atomic::Ordering;\n\n\
           type AtomicMaredPtr = ",
            stringify!($example_atomic_path),
            ";\ntype MarkedPtr = ",
            stringify!($example_ptr_path),
            ";\n\n\
           let reference = &mut 1;\n\
           let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b10));\n\
           assert_eq!(\n\
               \tptr.fetch_xor(0b01, Ordering::Relaxed).decompose(),\n\
               \t(reference as *mut _, 0b11)\n\
           );\n```"
        )
    };
    () => {
        "Performs a bitwise \"xor\" of `value` with the current tag value, returning the previous \
        marked pointer."
    };
}

macro_rules! doc_fetch_and {
    ($fn_ident:expr, $example_atomic_path:path, $example_ptr_path:path) => {
        concat!(
            doc_fetch_and!(),
            "\n\n",
            doc_fetch_and_x!("note"),
            "\n\n",
            doc_fetch_and_x!("ordering", $fn_ident),
            "\n\n# Examples\n\n\
           ```\nuse core::ptr;\n\
           use core::sync::atomic::Ordering;\n\n\
           type AtomicMaredPtr = ",
            stringify!($example_atomic_path),
            ";\ntype MarkedPtr = ",
            stringify!($example_ptr_path),
            ";\n\n\
           let reference = &mut 1;\n\
           let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b10));\n\
           assert_eq!(\n\
               \tptr.fetch_and(0b11, Ordering::Relaxed).decompose(),\n\
               \t(reference as *mut _, 0b10)\n\
           );\n```"
        )
    };
    () => {
        "Performs a bitwise \"and\" of `value` with the current tag value, returning the previous \
        marked pointer."
    };
}

macro_rules! doc_fetch_nand {
    ($fn_ident:expr, $example_atomic_path:path, $example_ptr_path:path) => {
        concat!(
            doc_fetch_nand!(),
            "\n\n",
            doc_fetch_and_x!("note"),
            "\n\n",
            doc_fetch_and_x!("ordering", $fn_ident),
            "\n\n# Examples\n\n\
           ```\nuse core::ptr;\n\
           use core::sync::atomic::Ordering;\n\n\
           type AtomicMaredPtr = ",
            stringify!($example_atomic_path),
            ";\ntype MarkedPtr = ",
            stringify!($example_ptr_path),
            ";\n\n\
           let reference = &mut 1;\n\
           let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b11));\n\
           assert_eq!(\n\
               \tptr.fetch_nand(0b11, Ordering::Relaxed).decompose(),\n\
               \t(reference as *mut _, 0b00)\n\
           );\n```"
        )
    };
    () => {
        "Performs a bitwise \"nand\" of `value` with the current tag value, returning the previous \
        marked pointer."
    };
}
