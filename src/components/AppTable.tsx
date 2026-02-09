import { GroupedRow } from "../types";
import { formatMs } from "../utils/stats";

type AppTableProps = {
  rows: GroupedRow[];
};

function AppTable({ rows }: AppTableProps) {
  return (
    <section className="card">
      <h2>按应用明细</h2>
      <div className="table">
        <div className="table-header">
          <span>应用</span>
          <span>打字</span>
          <span>按键</span>
          <span>会话</span>
        </div>
        {rows.length === 0 ? (
          <div className="table-empty">暂无数据</div>
        ) : (
          rows.map((row) => (
            <div className="table-row" key={row.app_name}>
              <span>{row.app_name}</span>
              <span>{formatMs(row.active_typing_ms)}</span>
              <span>{row.key_count}</span>
              <span>{row.session_count}</span>
            </div>
          ))
        )}
      </div>
    </section>
  );
}

export default AppTable;
