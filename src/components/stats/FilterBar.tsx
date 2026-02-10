import { Box, Button, ButtonGroup, Flex, Text } from "@chakra-ui/react";

type FilterBarProps = {
  filterRange: "today" | "yesterday" | "7d";
  onChange: (value: "today" | "yesterday" | "7d") => void;
};

function FilterBar({ filterRange, onChange }: FilterBarProps) {
  return (
    <Box bg="white" borderRadius="16px" p="5" boxShadow="sm">
      <Flex justify="space-between" align="center" gap="3" flexWrap="wrap">
        <Text fontWeight="semibold">时间范围</Text>
        <ButtonGroup size="sm" variant="outline" attached>
          <Button
            colorPalette={filterRange === "today" ? "blue" : "gray"}
            variant={filterRange === "today" ? "solid" : "outline"}
            onClick={() => onChange("today")}
          >
            今天
          </Button>
          <Button
            colorPalette={filterRange === "yesterday" ? "blue" : "gray"}
            variant={filterRange === "yesterday" ? "solid" : "outline"}
            onClick={() => onChange("yesterday")}
          >
            昨天
          </Button>
          <Button
            colorPalette={filterRange === "7d" ? "blue" : "gray"}
            variant={filterRange === "7d" ? "solid" : "outline"}
            onClick={() => onChange("7d")}
          >
            最近7天
          </Button>
        </ButtonGroup>
      </Flex>
    </Box>
  );
}

export default FilterBar;
