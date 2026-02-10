import { Box, Button, ButtonGroup, Flex, Text } from "@chakra-ui/react";

type FilterBarProps = {
  filterDays: 1 | 7;
  onChange: (value: 1 | 7) => void;
};

function FilterBar({ filterDays, onChange }: FilterBarProps) {
  return (
    <Box bg="white" borderRadius="16px" p="5" boxShadow="sm">
      <Flex justify="space-between" align="center" gap="3" flexWrap="wrap">
        <Text fontWeight="semibold">时间范围</Text>
        <ButtonGroup size="sm" variant="outline" attached>
          <Button
            colorPalette={filterDays === 1 ? "blue" : "gray"}
            variant={filterDays === 1 ? "solid" : "outline"}
            onClick={() => onChange(1)}
          >
            最近1天
          </Button>
          <Button
            colorPalette={filterDays === 7 ? "blue" : "gray"}
            variant={filterDays === 7 ? "solid" : "outline"}
            onClick={() => onChange(7)}
          >
            最近7天
          </Button>
        </ButtonGroup>
      </Flex>
    </Box>
  );
}

export default FilterBar;
