import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';

interface ServerControlProps {}

const ServerControl: React.FC<ServerControlProps> = () => {
    const [inputDevices, setInputDevices] = useState<string[]>([]);
    const [outputDevices, setOutputDevices] = useState<string[]>([]);
    const [selectedInputDevice, setSelectedInputDevice] = useState<string>('');
    const [selectedOutputDevice, setSelectedOutputDevice] = useState<string>('');
    const [isCapturing, setIsCapturing] = useState<boolean>(false);

    useEffect(() => {
        fetchAudioDevices();
    }, []);

    const fetchAudioDevices = async () => {
        try {
            console.log('Fetching audio devices...');
            const [inputs, outputs] = await invoke<[string[], string[]]>('get_audio_devices');
            setInputDevices(inputs);
            setOutputDevices(outputs);
            console.log('Fetched audio devices:', { inputs, outputs });
        } catch (error) {
            console.error('Failed to fetch audio devices:', error);
        }
    };

    const handleInputDeviceChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setSelectedInputDevice(event.target.value);
    };

    const handleOutputDeviceChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setSelectedOutputDevice(event.target.value);
    };

    const startCapture = async () => {
        try {
            console.log('Starting audio capture with:', { inputDevice: selectedInputDevice, outputDevice: selectedOutputDevice });
            const result = await invoke<string>('start_audio_capture', { inputDevice: selectedInputDevice, outputDevice: selectedOutputDevice });
            console.log(result);
            setIsCapturing(true);
            console.log('Audio capture started');
        } catch (error) {
            console.error('Failed to start audio capture:', error);
        }
    };

    return (
        <div>
            <div>
                <label>Dispositivo de Entrada:</label>
                <select value={selectedInputDevice} onChange={handleInputDeviceChange}>
                    <option value="">Selecione um dispositivo</option>
                    {inputDevices.map(device => (
                        <option key={device} value={device}>{device}</option>
                    ))}
                </select>
            </div>
            <div>
                <label>Dispositivo de Saída:</label>
                <select value={selectedOutputDevice} onChange={handleOutputDeviceChange}>
                    <option value="">Selecione um dispositivo</option>
                    {outputDevices.map(device => (
                        <option key={device} value={device}>{device}</option>
                    ))}
                </select>
            </div>
            <button onClick={startCapture} disabled={isCapturing || !selectedInputDevice || !selectedOutputDevice}>
                {isCapturing ? 'Capturando Áudio...' : 'Iniciar Captura de Áudio'}
            </button>
            <button onClick={fetchAudioDevices}>Atualizar Lista de Dispositivos</button>
        </div>
    );
};

export default ServerControl;
