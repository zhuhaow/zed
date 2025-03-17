use std::ptr::NonNull;

use crate::{FontFallbacks, FontFeatures};
use core_foundation::base::TCFType;
use font_kit::font::Font as FontKitFont;
use objc2::rc::Retained;
use objc2_core_foundation::{
    CFDictionary, CFLocaleCopyPreferredLanguages, CFRetained, CFString, Type,
};
use objc2_core_text::{
    kCTFontCascadeListAttribute, kCTFontFeatureSettingsAttribute, kCTFontOpenTypeFeatureTag,
    kCTFontOpenTypeFeatureValue, CTFont, CTFontCopyDefaultCascadeListForLanguages,
    CTFontCreateCopyWithAttributes, CTFontDescriptorCreateWithAttributes,
    CTFontDescriptorCreateWithNameAndSize,
};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSObject, NSString};

// This should be a util function but it's currently only needed here.
// https://github.com/madsmtm/objc2/issues/693
// Due to orphan rule, we cannot use `From` trait.
fn to_nsstring(cf_string: CFRetained<CFString>) -> Retained<NSString> {
    Into::<Retained<_>>::into(cf_string)
        .downcast::<NSString>()
        .unwrap()
}

fn generate_feature_array(features: &FontFeatures) -> Retained<NSArray<NSObject>> {
    let mut feature_array: Vec<Retained<NSObject>> = Vec::new();

    for (tag, value) in features.tag_value_list() {
        let dict = NSDictionary::<NSString, NSObject>::from_retained_objects::<NSString>(
            &[
                to_nsstring(unsafe { kCTFontOpenTypeFeatureTag.retain() }).as_ref(),
                to_nsstring(unsafe { kCTFontOpenTypeFeatureValue.retain() }).as_ref(),
            ],
            &[
                NSString::from_str(tag).downcast().unwrap(),
                NSNumber::new_u32(*value).downcast().unwrap(),
            ],
        );

        feature_array.push(dict.downcast().unwrap());
    }

    NSArray::from_retained_slice(&feature_array)
}

fn generate_fallback_array(
    fallbacks: &FontFallbacks,
    font: &CTFont,
) -> Retained<NSArray<NSObject>> {
    let mut fallback_array: Vec<Retained<NSObject>> = Vec::new();

    for name in fallbacks.fallback_list() {
        fallback_array.push(unsafe {
            Retained::cast_unchecked(
                CTFontDescriptorCreateWithNameAndSize(&CFString::from_str(name), 0.0).into(),
            )
        })
    }

    let preferred_languages = unsafe { CFLocaleCopyPreferredLanguages() };
    let default_fallbacks =
        unsafe { CTFontCopyDefaultCascadeListForLanguages(font, preferred_languages.as_deref()) };

    if let Some(default_fallbacks) = default_fallbacks {
        let default_fallbacks = Into::<Retained<_>>::into(default_fallbacks)
            .downcast::<NSArray>()
            .unwrap();
        for fallback in default_fallbacks.iter() {
            // TODO: check kCTFontURLAttribute exists
            fallback_array.push(unsafe { Retained::cast_unchecked(fallback) });
        }
    }

    NSArray::from_retained_slice(&fallback_array)
}

// TODO: Convert font to the right type by casting the pointer
pub fn apply_features_and_fallbacks(
    font: &mut FontKitFont,
    features: &FontFeatures,
    fallbacks: Option<&FontFallbacks>,
) -> anyhow::Result<()> {
    let feature_settings = to_nsstring(unsafe { kCTFontFeatureSettingsAttribute.retain() });
    let cascade_list = to_nsstring(unsafe { kCTFontCascadeListAttribute.retain() });

    let mut keys: Vec<&NSString> = vec![&feature_settings];
    let mut values = vec![generate_feature_array(features)];

    if let Some(fallbacks) = fallbacks {
        if !fallbacks.fallback_list().is_empty() {
            keys.push(&cascade_list);
            values.push(generate_fallback_array(fallbacks, unsafe {
                CFRetained::from_raw(NonNull::new_unchecked(
                    font.native_font().as_concrete_TypeRef() as *mut CTFont,
                ))
                .as_ref()
            }));
        }
    }

    let attrs = NSDictionary::<NSString, NSArray<NSObject>>::from_retained_objects(&keys, &values);

    let descriptor = unsafe {
        CTFontDescriptorCreateWithAttributes(
            CFRetained::from_raw(NonNull::new_unchecked(
                Retained::into_raw(attrs) as *mut CFDictionary
            ))
            .as_ref(),
        )
    };

    let new_font = unsafe {
        CTFontCreateCopyWithAttributes(
            CFRetained::from_raw(NonNull::new_unchecked(
                font.native_font().as_concrete_TypeRef() as *mut CTFont,
            ))
            .as_ref(),
            0.0,
            std::ptr::null(),
            Some(&descriptor),
        )
    };

    *font = unsafe {
        FontKitFont::from_native_font(&core_text::font::CTFont::wrap_under_create_rule(
            CFRetained::into_raw(new_font).as_ptr() as *mut core_text::font::__CTFont,
        ))
    };

    Ok(())
}
