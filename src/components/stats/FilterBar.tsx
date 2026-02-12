import { Button, HStack } from "@chakra-ui/react";

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

  return (
    <HStack bg="gray.100" p="1" borderRadius="10px" gap="0">
      {options.map((opt) => {
        const isActive = filterRange === opt.value;
        return (
          <Button
            key={opt.value}
            variant="ghost"
            size="sm"
            bg={isActive ? "white" : "transparent"}
            color={isActive ? "gray.900" : "gray.500"}
            shadow={isActive ? "xs" : "none"}
            borderRadius="8px"
            fontWeight={isActive ? "semibold" : "medium"}
            px="3"
            h="8"
            _hover={{
              bg: isActive ? "white" : "gray.200",
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
