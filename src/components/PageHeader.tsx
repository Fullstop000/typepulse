type PageHeaderProps = {
  title: string;
};

function PageHeader({ title }: PageHeaderProps) {
  return (
    <header className="header">
      <div>
        <h1>{title}</h1>
        <p className="subtle">需要授权输入监控权限以采集按键事件</p>
      </div>
    </header>
  );
}

export default PageHeader;
