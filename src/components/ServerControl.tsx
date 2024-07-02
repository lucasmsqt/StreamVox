import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';

interface ServerControlProps {
}

const ServerControl: React.FC<ServerControlProps> = () => {
    const [audioDevices, setAudioDevices] = useState<string[]>([]);
    const [selectedDevice, setSelectedDevice] = useState<string>('');
    const [isServerRunning, setIsServerRunning] = useState<boolean>(false);

    useEffect(() => {
        fetchAudioDevices();
    }, []);

    const fetchAudioDevices = async () => {
        try {
            const devices = await invoke('get_audio_devices');
            setAudioDevices(devices as string[]);
        } catch (error) {
            console.error('Failed to fetch audio devices:', error);
        }
    };

    const handleDeviceChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setSelectedDevice(event.target.value);
    };

    const toggleServer = () => {
        setIsServerRunning(!isServerRunning);
    };

    return (
        <div>
            <select value={selectedDevice} onChange={handleDeviceChange}>
                <option value="">Selecione um dispositivo</option>
                {audioDevices.map(device => (
                    <option key={device} value={device}>{device}</option>
                ))}
            </select>
            <button onClick={toggleServer}>
                {isServerRunning ? 'Desligar Servidor' : 'Ligar Servidor'}
            </button>
            <button onClick={fetchAudioDevices}>Atualizar Lista de Dispositivos</button>
        </div>
    );
};

export default ServerControl;
