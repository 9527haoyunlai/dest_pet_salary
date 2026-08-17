interface ErrorStateProps {
  message: string;
  onRetry: () => void;
}

export function ErrorState({ message, onRetry }: ErrorStateProps) {
  return (
    <section className="state-panel error-panel" role="alert">
      <p className="eyebrow">Connection interrupted</p>
      <h2>Salary Garden could not load</h2>
      <p>{message}</p>
      <button type="button" className="primary-button" onClick={onRetry}>
        Retry
      </button>
    </section>
  );
}
