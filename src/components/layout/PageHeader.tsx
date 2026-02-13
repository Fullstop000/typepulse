import { Heading } from "@chakra-ui/react";

type PageHeaderProps = {
  title: string;
};

function PageHeader({ title }: PageHeaderProps) {
  return <Heading size="2xl" fontWeight="bold" color="gray.800">{title}</Heading>;
}

export default PageHeader;
