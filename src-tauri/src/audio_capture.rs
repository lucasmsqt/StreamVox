extern crate winapi;
extern crate widestring;

use winapi::um::{
    combaseapi::{CoInitializeEx, CoCreateInstance, CoUninitialize},
    objbase::COINIT_APARTMENTTHREADED,
    coml2api::STGM_READ,
    functiondiscoverykeys_devpkey::PKEY_Device_FriendlyName,
    mmdeviceapi::{IMMDeviceEnumerator, CLSID_MMDeviceEnumerator, IMMDeviceCollection, IMMDevice, eRender, DEVICE_STATE_ACTIVE},
    propidl::{PROPVARIANT, PropVariantClear},
    combaseapi::CLSCTX_ALL,
    propsys::IPropertyStore,
};
use winapi::shared::winerror::S_OK;
use winapi::Interface;

use std::ptr;
use widestring::WideCString;

extern "C" {
    fn PropVariantToString(pv: *const PROPVARIANT, psz: *mut u16, cch: u32) -> i32;
}

pub fn list_audio_devices() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    unsafe {
        let hr = CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED);
        if hr != S_OK && hr != winapi::shared::winerror::S_FALSE {
            return Err(format!("Falha ao inicializar COM, código: 0x{:X}", hr).into());
        }

        let pclsid = CLSID_MMDeviceEnumerator;
        let mut enumerator: *mut IMMDeviceEnumerator = ptr::null_mut();
        let hr = CoCreateInstance(&pclsid, ptr::null_mut(), CLSCTX_ALL, &IMMDeviceEnumerator::uuidof(), &mut enumerator as *mut _ as *mut _);
        if hr != S_OK {
            CoUninitialize();
            return Err(format!("Falha ao criar instância do enumerador de dispositivos, código: 0x{:X}", hr).into());
        }

        let mut devices_ptr: *mut IMMDeviceCollection = ptr::null_mut();
        let hr = (*enumerator).EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE, &mut devices_ptr);
        if hr != S_OK {
            CoUninitialize();
            return Err(format!("Falha ao enumerar dispositivos de áudio, código: 0x{:X}", hr).into());
        }

        let devices = &*devices_ptr;
        let mut count = 0;
        if devices.GetCount(&mut count) != S_OK {
            CoUninitialize();
            return Err("Falha ao obter contagem de dispositivos".into());
        }

        let mut device_names = Vec::new();
        for i in 0..count {
            let mut device: *mut IMMDevice = ptr::null_mut();
            if devices.Item(i, &mut device) != S_OK {
                CoUninitialize();
                return Err("Falha ao obter dispositivo".into());
            }

            let mut property_store_ptr: *mut IPropertyStore = ptr::null_mut();
            if (*device).OpenPropertyStore(STGM_READ, &mut property_store_ptr) != S_OK {
                CoUninitialize();
                return Err("Falha ao abrir armazenamento de propriedade".into());
            }
            let property_store = &*property_store_ptr;

            let mut pv: PROPVARIANT = std::mem::zeroed();
            let key = PKEY_Device_FriendlyName;
            if property_store.GetValue(&key, &mut pv) != S_OK {
                CoUninitialize();
                return Err("Falha ao obter valor da propriedade".into());
            }

            let mut device_name_buf = vec![0u16; 256];
            if PropVariantToString(&pv, device_name_buf.as_mut_ptr(), device_name_buf.len() as u32) != S_OK {
                CoUninitialize();
                return Err("Falha ao converter PROPVARIANT para string".into());
            }

            let device_name = WideCString::from_ptr_str(device_name_buf.as_ptr()).to_string_lossy();
            device_names.push(device_name);
            PropVariantClear(&mut pv);
        }

        CoUninitialize();
        Ok(device_names)
    }
}
