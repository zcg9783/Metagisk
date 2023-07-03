use paste;
use std::ops::{Deref, DerefMut};

macro_rules! inherit {
    ($(struct $name:ident $(: $parent:ident)? {
        $($field:ident: $type:ty),* $(,)?
    })+) => {
        $(
            #[repr(C, align(4))]
            struct $name {
                $(base: $parent,)?
                $($field: $type),*
            }
            $(
                impl Deref for $name {
                    type Target = $parent;
                    fn deref(&self) -> &Self::Target {
                        &self.base
                    }
                }

                impl DerefMut for $name {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.base
                    }
                }
            )?
        )+
    };
}

macro_rules! dyn_enum {
    (
        enum $name:ident {
            $($variant:ident($variant_type:ty),)+
        }
        impl {
            $($var:ident : $($var_type:ident)+ @ $($ver:ident)|*,)*
        }
    ) => {
        enum $name {
            $($variant{hdr: $variant_type, j32: u32, j64: u64},)*
        }
        impl $name {
            fn junk_u32(&self) -> &u32 {
                &0
            }
            fn junk_u64(&self) -> &u64 {
                &0
            }
            fn junk_char_seq(&self) -> &CharSeq {
                &[]
            }
            fn junk_u32_mut(&mut self) -> &mut u32 {
                match self {
                    $(
                        Self::$variant{j32, ..} => j32,
                    )*
                }
            }
            fn junk_u64_mut(&mut self) -> &mut u64 {
                match self {
                    $(
                        Self::$variant{j64, ..} => j64,
                    )*
                }
            }
            dyn_enum!(@decl_fields $($var : $($var_type)+ @ $($ver)|*),* @ ($($variant),*));
        }
    };
    (@decl_fields $($var:ident : $($var_type:ident)+ @ $($ver:ident)|*),* @ $tuple:tt) => {
        $(
            dyn_enum!(@decl_field $var: $($var_type)+ @ $($ver)|*, @ $tuple);
        )*
    };
    (@decl_field $var:ident: $var_type:ident @ $($ver:ident)|*, @ ($($variant:ident),*)) => {
        fn $var(&self) -> &$var_type {
            match self {
                $(
                    Self::$ver{hdr, ..} => &hdr.$var,
                )*
                #[allow(unreachable_patterns)]
                _ => paste::item! { self.[< junk_ $var_type:snake >]() },
            }
        }
        dyn_enum!(@decl_mut_str $var: $var_type @ $($ver)|*, @ ($($variant),*));
    };
    (@decl_field $var:ident: mut $var_type:ident @ $($ver:ident)|*, @ ($($variant:ident),*)) => {
        paste::item! {
            fn [< $var _mut >](&mut self) -> &mut $var_type {
                match self {
                    $(
                        Self::$ver{hdr, ..} => &mut hdr.$var,
                    )*
                    #[allow(unreachable_patterns)]
                    _ => self.[< junk_ $var_type _mut >](),
                }
            }
        }
        dyn_enum!(@decl_field $var: $var_type @ $($ver)|*, @ ($($variant),*));
    };
    (@decl_mut_str $var:ident: CharSeq @ $($ver:ident)|*, @ ($($variant:ident),*)) => {
        paste::item! {
            fn [< $var _mut >](&mut self) -> Option<&mut CharSeq> {
                match self {
                    $(
                        Self::$ver{hdr, ..} => Some(&mut hdr.$var),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => None
                }
            }
        }
    };

    (@decl_mut_str $var:ident: $var_type:ident @ $($ver:ident)|*, @ ($($variant:ident),*)) => {
    };
}

const BOOT_MAGIC_SIZE: usize = 8;
const BOOT_NAME_SIZE: usize = 16;
const BOOT_ID_SIZE: usize = 32;
const BOOT_ARGS_SIZE: usize = 512;
const BOOT_EXTRA_ARGS_SIZE: usize = 1024;
const VENDOR_BOOT_ARGS_SIZE: usize = 2048;
const VENDOR_RAMDISK_NAME_SIZE: usize = 32;
const VENDOR_RAMDISK_TABLE_ENTRY_BOARD_ID_SIZE: usize = 16;

inherit! {
    struct BootImgHdrCommon {
        magic: [u8; BOOT_MAGIC_SIZE],
        kernel_size: u32,
        kernel_addr: u32,
        ramdisk_size: u32,
        ramdisk_addr: u32,
        second_size: u32,
        second_addr: u32,
    }
    struct BootImgHdrPxa : BootImgHdrCommon {
        extra_size: u32,
        unknown: u32,
        tags_addr: u32,
        page_size: u32,
        name: [char; 24],
        cmdline: [char; BOOT_ARGS_SIZE],
        id: [char; BOOT_ID_SIZE],
        extra_cmdline: [char; BOOT_EXTRA_ARGS_SIZE],
    }
    struct BootImgHdrV0 : BootImgHdrCommon {
        tags_addr: u32,
        page_size: u32,
        header_version: u32,
        os_version: u32,
        name: [char; BOOT_NAME_SIZE],
        cmdline: [char; BOOT_ARGS_SIZE],
        id: [char; BOOT_ID_SIZE],
        extra_cmdline: [char; BOOT_EXTRA_ARGS_SIZE],
    }
    struct BootImgHdrV1 : BootImgHdrV0 {
        recovery_dtbo_size: u32,
        recovery_dtbo_offset: u64,
        header_size: u32,
    }
    struct BootImgHdrV2 : BootImgHdrV1 {
        dtb_size: u32,
        dtb_addr: u64,
    }
    struct BootImgHdrV3 {
        magic: [u8; BOOT_MAGIC_SIZE],
        kernel_size: u32,
        ramdisk_size: u32,
        os_version: u32,
        header_size: u32,
        reserved: [u32; 4],
        header_version: u32,
        cmdline: [char; BOOT_ARGS_SIZE + BOOT_EXTRA_ARGS_SIZE],
    }
    struct BootImgHdrV4 : BootImgHdrV3 {
        signature_size: u32,
    }
    struct BootImgHdrVndV3 {
        magic: [u8; BOOT_MAGIC_SIZE],
        header_version: u32,
        page_size: u32,
        kernel_addr: u32,
        ramdisk_addr: u32,
        ramdisk_size: u32,
        cmdline: [char; VENDOR_BOOT_ARGS_SIZE],
        tags_addr: u32,
        name: [char; BOOT_NAME_SIZE],
        header_size: u32,
        dtb_size: u32,
        dtb_addr: u64,
    }
    struct BootImgHdrVndV4 : BootImgHdrVndV3 {
        vendor_ramdisk_size: u32,
        vendor_ramdisk_table_entry_num: u32,
        vendor_ramdisk_table_entry_size: u32,
        bootconfig_size: u32,
    }
}

type CharSeq = [char];

dyn_enum! {
    enum DynImg {
        V0(BootImgHdrV0),
        V1(BootImgHdrV1),
        V2(BootImgHdrV2),
        V3(BootImgHdrV3),
        V4(BootImgHdrV4),
        VndV3(BootImgHdrVndV3),
        VndV4(BootImgHdrVndV4),
        Pxa(BootImgHdrPxa),
    }
    impl {
        kernel_size: mut u32 @ V0|V1|V2|V3|V4|Pxa,
        ramdisk_size: mut u32 @ V0|V1|V2|V3|V4|VndV3|VndV4|Pxa,
        second_size: mut u32 @ V0|V1|V2|Pxa,
        page_size: u32 @ V0|V1|V2|VndV3|VndV4|Pxa,
        os_version: mut u32 @ V0|V1|V2|V3|V4,
        name: CharSeq @ V0|V1|V2|VndV3|VndV4|Pxa,
        cmdline: CharSeq @ V0|V1|V2|V3|V4|VndV3|VndV4|Pxa,
        id: CharSeq @ V0|V1|V2|Pxa,
        extra_cmdline: CharSeq @ V0|V1|V2|Pxa,
        header_version: u32 @ V1|V2|V3|V4|VndV3|VndV4,
        recovery_dtbo_size: u32 @ V1|V2,
        recovery_dtbo_offset: u64 @ V1|V2,
        header_size: u32 @ V1|V2|V3|V4,
        dtb_size: u32 @ V2|VndV3|VndV4,
        signature_size: u32 @ V4,
        vendor_ramdisk_size: u32 @ VndV4,
        bootconfig_size: u32 @ VndV4,
    }
}

impl DynImg {
    fn extra_size(&self) -> &u32 {
        match self {
            Self::V0 {
                hdr: BootImgHdrV0 { header_version, .. },
                ..
            } => header_version,
            Self::Pxa {
                hdr: BootImgHdrPxa { extra_size, .. },
                ..
            } => extra_size,
            _ => self.junk_u32(),
        }
    }
    fn extra_size_mut(&mut self) -> &mut u32 {
        match self {
            Self::V0 {
                hdr:
                    BootImgHdrV0 {
                        ref mut header_version,
                        ..
                    },
                ..
            } => header_version,
            Self::Pxa {
                hdr: BootImgHdrPxa {
                    ref mut extra_size, ..
                },
                ..
            } => extra_size,
            _ => self.junk_u32_mut(),
        }
    }
}
