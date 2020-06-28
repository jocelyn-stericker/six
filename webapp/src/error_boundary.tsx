import React from "react";

interface Props {
  fallback: (err: Error, clearErr: () => unknown) => React.ReactNode;
  children: React.ReactNode;
}

export default class ErrorBoundary extends React.Component<Props> {
  state: { err: Error | null };

  constructor(props: Props) {
    super(props);
    this.state = { err: null };
  }

  componentDidCatch(err: Error) {
    this.setState({ err });
  }

  clearErr() {
    this.setState({ err: null });
  }

  render(): React.ReactNode {
    return this.state.err
      ? this.props.fallback(this.state.err, this.clearErr)
      : this.props.children;
  }
}
