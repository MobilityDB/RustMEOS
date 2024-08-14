use crate::{
    boxes::tbox::TBox,
    collections::number::{number_span::NumberSpan, number_span_set::NumberSpanSet},
    temporal::temporal::Temporal,
};

pub trait TNumber: Temporal<TBB = TBox> {
    // ------------------------- Accessors -------------------------------------
    /// Returns the bounding box of `self`.
    ///
    /// # Returns
    /// The bounding box of `self`.
    ///
    /// # Safety
    /// This function uses unsafe code to call the `meos_sys::tbox_tnumber` function.
    fn bounding_box(&self) -> TBox {
        unsafe { TBox::from_inner(meos_sys::tnumber_to_tbox(self.inner())) }
    }

    /// Returns the integral of `self`.
    ///
    /// # Returns
    /// The integral of `self`.
    ///
    /// # Safety
    /// This function uses unsafe code to call the `meos_sys::tnumber_integral` function.
    fn integral(&self) -> f64 {
        unsafe { meos_sys::tnumber_integral(self.inner()) }
    }

    /// Returns the time-weighted average of `self`.
    ///
    /// # Returns
    /// The time-weighted average of `self`.
    ///
    /// # Safety
    /// This function uses unsafe code to call the `meos_sys::tnumber_twavg` function.
    fn time_weighted_average(&self) -> f64 {
        unsafe { meos_sys::tnumber_twavg(self.inner()) }
    }

    // ------------------------- Restrictions ----------------------------------
    /// Returns a new temporal object with the values of `self` where it's not in `span`
    ///
    /// ## Arguments
    /// * `span` - A `IntSpan` or `FloatSpan` to substract the values from
    fn minus_span(&self, span: impl NumberSpan) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::tnumber_minus_span(self.inner(), span.inner())
        })
    }

    /// Returns a new temporal object with the values of `self` where it's not in `span_set`
    ///
    /// ## Arguments
    /// * `span_set` - A `IntSpanSet` or `FloatSpanSet` to substract the values from
    fn minus_span_set(&self, span_set: impl NumberSpanSet) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::tnumber_minus_spanset(self.inner(), span_set.inner())
        })
    }

    // ------------------------- Operations ------------------------------------
    /// Adds the value(s) of `other` to the value(s) of `self`.
    ///
    /// # Arguments
    /// * `other` - A temporal number to add to the value(s) of `self`.
    fn add(&self, other: &Self) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::add_tnumber_tnumber(self.inner(), other.inner())
        })
    }

    /// Substract the value(s) of `other` to the value(s) of `self`.
    ///
    /// # Arguments
    /// * `other` - A temporal number to substract to the value(s) of `self`.
    fn substract(&self, other: &Self) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::sub_tnumber_tnumber(self.inner(), other.inner())
        })
    }

    /// Multiplicate the value(s) of `other` by the value(s) of `self`.
    ///
    /// # Arguments
    /// * `other` - A temporal number to multiplicate by the value(s) of `self`.
    fn multiplicate(&self, other: &Self) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::mult_tnumber_tnumber(self.inner(), other.inner())
        })
    }

    /// Divide the value(s) of `other` by the value(s) of `self`.
    ///
    /// # Arguments
    /// * `other` - A temporal number to divide by the value(s) of `self`.
    fn divide(&self, other: &Self) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::div_tnumber_tnumber(self.inner(), other.inner())
        })
    }

    // ------------------------- Aggregations ----------------------------------
    /// Returns the absolute value of `self`.
    ///
    /// # Returns
    /// The absolute value of `self`.
    fn abs(&self) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::tnumber_abs(self.inner()) })
    }

    /// Returns the change in value between successive pairs of `self`.
    ///
    /// # Returns
    /// The change in value between successive pairs of `self`.
    fn delta_value(&self) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::tnumber_delta_value(self.inner()) })
    }

    /// Returns the distance between `self` and `other`.
    ///
    /// # Arguments
    /// * `other` - A temporal number to compute the distance to.
    fn distance(&self, other: &Self) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::distance_tnumber_tnumber(self.inner(), other.inner())
        })
    }

    /// Returns the nearest approach distance between `self` and `other`.
    ///
    /// # Arguments
    /// * `other` - A temporal number to compute the nearest approach distance to.
    fn nearest_approach_distance(&self, other: &Self) -> Self::Type;
}

/// Generates the neccessary code to implement the temporal trait for the appropriate type
///
/// ## Parameters:
///    - `type`: The actual Rust type to implement the traits to
///    - `temporal_type`: Whether it's Instant, Sequence, or SequenceSet
///    - `base_type`: The base Rust type, i32 or f64.
///    - `basic_type`: Whether it's Int or Float.
macro_rules! impl_temporal_for_tnumber {
    ($type:ty, $temporal_type:ident, $base_type:ty, $basic_type:ident) => {
        paste::paste! {
            impl Collection for $type {
                impl_collection!(tnumber, $base_type);

                fn contains(&self, content: &Self::Type) -> bool {
                    [<$basic_type SpanSet>]::from_inner(unsafe { meos_sys::tnumber_valuespans(self.inner()) })
                        .contains(content)
                }
            }
            impl_simple_traits_for_temporal!($type, [<t $basic_type:lower>]);


            impl TNumber for $type {
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type {
                    unsafe { meos_sys::[<nad_ t $basic_type:lower _ t $basic_type:lower>](self.inner(), other.inner()) }
                }
            }

            impl OrderedTemporal for $type {
                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<t $basic_type:lower _min_value>](self.inner()) }
                }

                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<t $basic_type:lower _max_value>](self.inner()) }
                }

                impl_ordered_temporal_functions!([<$basic_type:lower>]);
            }
            impl Temporal for $type {
                type TI = [<T $basic_type Instant>];
                type TS = [<T $basic_type Sequence>];
                type TSS = [<T $basic_type SequenceSet>];
                type TBB = TBox;
                type Enum = [<T $basic_type>];
                type TBoolType = [<TBool $temporal_type>];

                fn from_inner_as_temporal(inner: *mut meos_sys::Temporal) -> Self {
                    Self {
                        _inner: ptr::NonNull::new(inner as *mut meos_sys::[<T $temporal_type>]).expect("Null pointers not allowed"),
                    }
                }

                fn inner(&self) -> *const meos_sys::Temporal {
                    self._inner.as_ptr() as *const meos_sys::Temporal
                }

                fn bounding_box(&self) -> Self::TBB {
                    TNumber::bounding_box(self)
                }

                fn values(&self) -> Vec<Self::Type> {
                    let mut count = 0;
                    unsafe {
                        let values = meos_sys::[<t $basic_type:lower _values>](self.inner(), ptr::addr_of_mut!(count));

                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }

                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<t $basic_type:lower _start_value>](self.inner()) }
                }

                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<t $basic_type:lower _end_value>](self.inner()) }
                }

                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0.into();
                    unsafe {
                        let success = meos_sys::[<t $basic_type:lower _value_at_timestamptz>](
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            ptr::addr_of_mut!(result),
                        );
                        if success {
                            Some(result)
                        } else {
                            None
                        }
                    }
                }

                fn at_value(&self, value: &Self::Type) -> Option<Self::Enum> {
                    let result = unsafe { meos_sys::[<t $basic_type:lower _at_value>](self.inner(), *value) };
                    if !result.is_null() {
                        Some(factory::<Self::Enum>(result))
                    } else {
                        None
                    }
                }

                fn at_values(&self, values: &[Self::Type]) -> Option<Self::Enum> {
                    unsafe {
                        let set = meos_sys::[<$basic_type:lower set_make>](values.as_ptr(), values.len() as i32);
                        let result = meos_sys::temporal_at_values(self.inner(), set);
                        if !result.is_null() {
                            Some(factory::<Self::Enum>(result))
                        } else {
                            None
                        }
                    }
                }

                fn minus_value(&self, value: Self::Type) -> Self::Enum {
                    factory::<Self::Enum>(unsafe {
                        meos_sys::[<t $basic_type:lower _minus_value>](self.inner(), value)
                    })
                }

                fn minus_values(&self, values: &[Self::Type]) -> Self::Enum {
                    factory::<Self::Enum>(unsafe {
                        let set = meos_sys::[<$basic_type:lower set_make>](values.as_ptr(), values.len() as i32);
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }

                fn temporal_equal_value(&self, value: &Self::Type) -> Self::TBoolType {
                    Self::TBoolType::from_inner_as_temporal(unsafe {
                        meos_sys::[<teq_t $basic_type:lower _ $basic_type:lower>](self.inner(), *value)
                    })
                }

                fn temporal_not_equal_value(&self, value: &Self::Type) -> Self::TBoolType {
                    Self::TBoolType::from_inner_as_temporal(unsafe {
                        meos_sys::[<tne_t $basic_type:lower _ $basic_type:lower>](self.inner(), *value)
                    })
                }

                impl_always_and_ever_value_equality_functions!([<$basic_type:lower>]);
            }
        }
    }
}

pub(crate) use impl_temporal_for_tnumber;
