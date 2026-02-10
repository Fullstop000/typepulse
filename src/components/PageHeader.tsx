type PageHeaderProps = {
  title: string;
};

function PageHeader({ title }: PageHeaderProps) {
  return (
    <header className="header">
      <div>
        <h1>{title}</h1>
      </div>
    </header>
  );
}

export default PageHeader;
