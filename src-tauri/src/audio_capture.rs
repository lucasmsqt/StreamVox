extern crate winapi;
extern crate widestring;
extern crate cpal;
extern crate rodio;

use winapi::um::{
    combaseapi::{CoInitializeEx, CoCreateInstance, CoUninitialize},
    objbase::COINIT_APARTMENTTHREADED,
    coml2api::STGM_READ,
    functiondiscoverykeys_devpkey::PKEY_Device_FriendlyName,
    mmdeviceapi::{IMMDeviceEnumerator, CLSID_MMDeviceEnumerator, IMMDeviceCollection, IMMDevice, eRender, eCapture, DEVICE_STATE_ACTIVE},
    propidl::{PROPVARIANT, PropVariantClear},
    combaseapi::CLSCTX_ALL,
    propsys::IPropertyStore,
};
use winapi::shared::winerror::S_OK;
use winapi::Interface;

use std::ptr;
use widestring::WideCString;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::{OutputStream, Sink};
use std::sync::{Arc, Mutex};
use std::thread;

extern "C" {
    fn PropVariantToString(pv: *const PROPVARIANT, psz: *mut u16, cch: u32) -> i32;
}

pub fn list_audio_devices() -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
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

        // Enumerar dispositivos de entrada (captura)
        let mut capture_devices_ptr: *mut IMMDeviceCollection = ptr::null_mut();
        let hr = (*enumerator).EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE, &mut capture_devices_ptr);
        if hr != S_OK {
            CoUninitialize();
            return Err(format!("Falha ao enumerar dispositivos de áudio de captura, código: 0x{:X}", hr).into());
        }

        let capture_devices = &*capture_devices_ptr;
        let mut capture_count = 0;
        if capture_devices.GetCount(&mut capture_count) != S_OK {
            CoUninitialize();
            return Err("Falha ao obter contagem de dispositivos de captura".into());
        }

        let mut capture_device_names = Vec::new();
        for i in 0..capture_count {
            let mut device: *mut IMMDevice = ptr::null_mut();
            if capture_devices.Item(i, &mut device) != S_OK {
                CoUninitialize();
                return Err("Falha ao obter dispositivo de captura".into());
            }
            if let Some(name) = get_device_name(device) {
                capture_device_names.push(name);
            }
        }

        // Enumerar dispositivos de saída (renderização)
        let mut render_devices_ptr: *mut IMMDeviceCollection = ptr::null_mut();
        let hr = (*enumerator).EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE, &mut render_devices_ptr);
        if hr != S_OK {
            CoUninitialize();
            return Err(format!("Falha ao enumerar dispositivos de áudio de renderização, código: 0x{:X}", hr).into());
        }

        let render_devices = &*render_devices_ptr;
        let mut render_count = 0;
        if render_devices.GetCount(&mut render_count) != S_OK {
            CoUninitialize();
            return Err("Falha ao obter contagem de dispositivos de renderização".into());
        }

        let mut render_device_names = Vec::new();
        for i in 0..render_count {
            let mut device: *mut IMMDevice = ptr::null_mut();
            if render_devices.Item(i, &mut device) != S_OK {
                CoUninitialize();
                return Err("Falha ao obter dispositivo de renderização".into());
            }
            if let Some(name) = get_device_name(device) {
                render_device_names.push(name);
            }
        }

        CoUninitialize();
        Ok((capture_device_names, render_device_names))
    }
}

unsafe fn get_device_name(device: *mut IMMDevice) -> Option<String> {
    let mut property_store_ptr: *mut IPropertyStore = ptr::null_mut();
    if (*device).OpenPropertyStore(STGM_READ, &mut property_store_ptr) != S_OK {
        return None;
    }
    let property_store = &*property_store_ptr;

    let mut pv: PROPVARIANT = std::mem::zeroed();
    let key = PKEY_Device_FriendlyName;
    if property_store.GetValue(&key, &mut pv) != S_OK {
        return None;
    }

    let mut device_name_buf = vec![0u16; 256];
    if PropVariantToString(&pv, device_name_buf.as_mut_ptr(), device_name_buf.len() as u32) != S_OK {
        return None;
    }

    let device_name = WideCString::from_ptr_str(device_name_buf.as_ptr()).to_string_lossy();
    PropVariantClear(&mut pv);
    Some(device_name)
}

pub fn capture_and_play_audio(input_device_name: &str, output_device_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    // Selecionar dispositivo de entrada
    let input_device = host.input_devices()?
        .find(|d| d.name().unwrap_or_default() == input_device_name)
        .ok_or("Dispositivo de entrada não encontrado")?;
    
    println!("Input device found: {:?}", input_device.name());

    // Selecionar dispositivo de saída
    let output_device = host.output_devices()?
        .find(|d| d.name().unwrap_or_default() == output_device_name)
        .ok_or("Dispositivo de saída não encontrado")?;
    
    println!("Output device found: {:?}", output_device.name());

    let input_config = input_device.default_input_config()?;
    let output_config = output_device.default_output_config()?;

    println!("Input config: {:?}", input_config);
    println!("Output config: {:?}", output_config);

    if input_config.channels() != output_config.channels() {
        return Err("Número de canais de entrada e saída não correspondem".into());
    }

    let (_stream, stream_handle) = OutputStream::try_from_device(&output_device)?;
    let sink = Sink::try_new(&stream_handle)?;
    let sink = Arc::new(Mutex::new(sink));

    let sink_clone = Arc::clone(&sink);
    let input_config_clone = input_config.clone();
    let err_fn = move |err| eprintln!("An error occurred on the input audio stream: {}", err);

    let stream = input_device.build_input_stream(
        &input_config.clone().into(),
        move |data: &[f32], _: &_| {
            if data.is_empty() {
                println!("No audio data received.");
            } else {
                println!("Audio data received: {} samples", data.len());
            }
            let source = rodio::buffer::SamplesBuffer::new(input_config_clone.channels() as u16, input_config_clone.sample_rate().0, data.to_vec());
            println!("Audio data being appended to sink.");
            let mut sink_guard = sink_clone.lock().unwrap();
            sink_guard.append(source);
            println!("Audio data appended to sink.");
            if sink_guard.is_paused() {
                sink_guard.play();
                println!("Sink is playing.");
            }
        },
        err_fn,
    )?;

    stream.play()?;
    println!("Audio stream started.");

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}

