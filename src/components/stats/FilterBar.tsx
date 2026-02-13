import { Box, Button, HStack } from "@chakra-ui/react";

type FilterBarProps = {
  filterRange: "today" | "yesterday" | "7d";
  onChange: (value: "today" | "yesterday" | "7d") => void;
};

function FilterBar({ filterRange, onChange }: FilterBarProps) {
  const options = [
    { value: "today", label: "今天" },
    { value: "yesterday", label: "昨天" },
    { value: "7d", label: "最近7天" },
  ] as const;
  const activeIndex = options.findIndex((opt) => opt.value === filterRange);

  return (
    <HStack
      position="relative"
      bg="glass.pill"
      borderWidth="1px"
      borderColor="glass.border"
      backdropFilter="blur(10px) saturate(1.05)"
      css={{ WebkitBackdropFilter: "blur(10px) saturate(1.05)" }}
      p="1"
      borderRadius="999px"
      gap="0"
      overflow="hidden"
    >
      {/* Sliding highlight gives iOS-like smooth segment movement. */}
      <Box
        position="absolute"
        left="1"
        top="1"
        bottom="1"
        w={`calc((100% - 8px) / ${options.length})`}
        borderRadius="999px"
        bg="rgba(255, 255, 255, 0.82)"
        borderWidth="1px"
        borderColor="rgba(255,255,255,0.75)"
        boxShadow="0 3px 14px rgba(15, 23, 42, 0.10), inset 0 1px 0 rgba(255,255,255,0.55)"
        transform={`translateX(${Math.max(activeIndex, 0) * 100}%)`}
        transition="transform 340ms cubic-bezier(0.22, 1, 0.36, 1)"
        pointerEvents="none"
      />
      {options.map((opt) => {
        const isActive = filterRange === opt.value;
        return (
          <Button
            key={opt.value}
            variant="ghost"
            size="sm"
            bg="transparent"
            color={isActive ? "gray.900" : "gray.600"}
            borderRadius="999px"
            fontWeight={isActive ? "semibold" : "medium"}
            px="3.5"
            h="8"
            flex="1"
            position="relative"
            zIndex={1}
            _hover={{
              bg: "transparent",
              color: isActive ? "gray.900" : "gray.700",
            }}
            onClick={() => onChange(opt.value)}
          >
            {opt.label}
          </Button>
        );
      })}
    </HStack>
  );
}

export default FilterBar;
