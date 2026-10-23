import { RotateCcw, TriangleAlert } from "lucide-react";
import { Component, type ReactNode } from "react";

/** Keeps one broken screen from blanking the whole app. */
export class ErrorBoundary extends Component<{ children: ReactNode; label: string }, { error: Error | null }> {
  state: { error: Error | null } = { error: null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="crash" role="alert">
        <TriangleAlert size={22} />
        <div>
          <b className="caps">{this.props.label} failed to load</b>
          <p className="muted">{this.state.error.message}</p>
        </div>
        <button className="hud-btn" onClick={() => this.setState({ error: null })}>
          <RotateCcw size={14} /> Try again
        </button>
      </div>
    );
  }
}
