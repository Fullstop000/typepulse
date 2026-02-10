import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function StorageSettingsSection() {
  const [dataSize, setDataSize] = useState<number | null>(null);

  const handleOpenDataDir = async () => {
    await invoke("open_data_dir");
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) {
      return `${bytes} B`;
    }
    const units = ["KB", "MB", "GB", "TB"];
    let size = bytes;
    let index = -1;
    while (size >= 1024 && index < units.length - 1) {
      size /= 1024;
      index += 1;
    }
    const precision = size >= 10 || index === 0 ? 0 : 1;
    return `${size.toFixed(precision)} ${units[index]}`;
  };

  useEffect(() => {
    let mounted = true;
    invoke<number>("get_data_dir_size")
      .then((size) => {
        if (mounted) {
          setDataSize(size);
        }
      })
      .catch(() => {
        if (mounted) {
          setDataSize(null);
        }
      });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <section className="card">
      <h2>数据存储</h2>
      <p className="subtle">数据与日志保存在本机应用数据目录。</p>
      {dataSize !== null ? <p className="subtle">已用空间：{formatBytes(dataSize)}</p> : null}
      <div className="actions">
        <button onClick={handleOpenDataDir}>前往数据目录</button>
      </div>
    </section>
  );
}

export default StorageSettingsSection;
