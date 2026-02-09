type FilterBarProps = {
  filterDays: 1 | 7;
  onChange: (value: 1 | 7) => void;
};

function FilterBar({ filterDays, onChange }: FilterBarProps) {
  return (
    <section className="card filter-bar">
      <div className="label">时间范围</div>
      <div className="row">
        <button
          onClick={() => onChange(1)}
          className={filterDays === 1 ? "tab active" : "tab"}
        >
          最近1天
        </button>
        <button
          onClick={() => onChange(7)}
          className={filterDays === 7 ? "tab active" : "tab"}
        >
          最近7天
        </button>
      </div>
    </section>
  );
}

export default FilterBar;
