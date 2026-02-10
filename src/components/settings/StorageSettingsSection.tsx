import { invoke } from "@tauri-apps/api/core";
import { Box, Button, Text } from "@chakra-ui/react";
import { useEffect, useState } from "react";

function StorageSettingsSection() {
  const [dataSize, setDataSize] = useState<number | null>(null);

  const handleOpenDataDir = async () => {
    await invoke("open_data_dir");
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
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
        if (mounted) setDataSize(size);
      })
      .catch(() => {
        if (mounted) setDataSize(null);
      });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Text fontSize="xl" fontWeight="semibold" mb="2">数据存储</Text>
      <Text fontSize="sm" color="gray.600" mb="1">数据与日志保存在本机应用数据目录。</Text>
      {dataSize !== null ? (
        <Text fontSize="sm" color="gray.600" mb="4">已用空间：{formatBytes(dataSize)}</Text>
      ) : null}
      <Button onClick={handleOpenDataDir}>前往数据目录</Button>
    </Box>
  );
}

export default StorageSettingsSection;
