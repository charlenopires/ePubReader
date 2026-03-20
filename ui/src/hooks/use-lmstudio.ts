import { useState, useEffect, useCallback } from 'react';
import { api } from '@/lib/tauri';
import { useAppStore } from '@/stores/app-store';

export function useLmStudio() {
  const [checking, setChecking] = useState(true);
  const { lmstudioConnected, availableModels, currentModel, setLmStudioStatus, setCurrentModel } = useAppStore();

  const checkStatus = useCallback(async () => {
    try {
      setChecking(true);
      const status = await api.checkLmStudioStatus();
      setLmStudioStatus(status.is_running, status.available_models);
      if (status.is_running) {
        try {
          const model = await api.getCurrentModel();
          setCurrentModel(model);
        } catch {
          if (status.available_models.length > 0) {
            setCurrentModel(status.available_models[0]);
          }
        }
      }
    } catch {
      setLmStudioStatus(false, []);
    } finally {
      setChecking(false);
    }
  }, [setLmStudioStatus, setCurrentModel]);

  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  const setModel = useCallback(async (model: string) => {
    try {
      await api.setModel(model);
      setCurrentModel(model);
    } catch (err) {
      console.error('Failed to set model:', err);
    }
  }, [setCurrentModel]);

  return { connected: lmstudioConnected, checking, availableModels, currentModel, checkStatus, setModel };
}
