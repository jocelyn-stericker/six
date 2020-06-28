import {
  Dispatch,
  Reducer,
  ReducerAction,
  ReducerState,
  useEffect,
  useReducer,
} from "react";

export function useLocallyPersistedReducer<R extends Reducer<any, any>, I>(
  reducer: R,
  defaultState: I,
  storageKey: string,
  init: (arg: I) => ReducerState<R>,
  loader: (x: string) => any,
): [ReducerState<R>, Dispatch<ReducerAction<R>>] {
  const hookVars = useReducer(reducer, defaultState, defaultState => {
    const string = localStorage.getItem(storageKey);
    const persisted = string ? loader(string) : null;
    if (persisted === null) {
      if (init === null) {
        return defaultState;
      } else {
        return init(defaultState);
      }
    } else {
      return persisted;
    }
  });

  const firstHookVar = hookVars[0];
  useEffect(() => {
    localStorage.setItem(storageKey, JSON.stringify(firstHookVar));
  }, [storageKey, firstHookVar]);

  return hookVars;
}
