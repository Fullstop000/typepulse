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
    <Box bg="#f5f5f6" borderRadius="12px" borderWidth="1px" borderColor="#dddddf" p="0" overflow="hidden">
      <Box px="5" py="4" borderBottomWidth="1px" borderColor="#e4e4e7">
        <Text fontSize="lg" fontWeight="semibold" color="#111827">Storage</Text>
      </Box>
      <Box px="5" py="4">
        <Text fontSize="sm" color="#6b7280" mb="1">数据与日志保存在本机应用数据目录。</Text>
        {dataSize !== null ? (
          <Text fontSize="sm" color="#6b7280" mb="4">已用空间：{formatBytes(dataSize)}</Text>
        ) : null}
        <Button
          onClick={handleOpenDataDir}
          bg="#e7e7ea"
          color="#1f2328"
          borderWidth="1px"
          borderColor="#d1d5db"
          _hover={{ bg: "#dddddf" }}
        >
          前往数据目录
        </Button>
      </Box>
    </Box>
  );
}

export default StorageSettingsSection;
