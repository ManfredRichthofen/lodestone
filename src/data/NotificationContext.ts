import { getSnowflakeTimestamp } from './../utils/util';
import { ClientEvent } from 'bindings/ClientEvent';
import { createContext, useReducer } from 'react';
import { ProgressionEvent } from 'bindings/ProgressionEvent';
import { match } from 'variant';

export type NotificationStatus = 'error' | 'info' | 'success';

export type NotificationItem = {
  status: NotificationStatus;
  message: string;
  timestamp: number;
  key: string;
};

export type OngoingState = 'ongoing' | 'done' | 'error';

// Invariant progress = parent
export type OngoingNotificationItem = {
  state: OngoingState;
  progress: number;
  total: number | null;
  title: string;
  message: string | null;
  timestamp: number;
  event_id: string;
  key: string;
};

// used for dispatching to the notification reducer
export type NotificationAction = {
  message: string;
  status: NotificationStatus;
  event: ClientEvent;
};

export type OngoingNotificationAction = {
  event: ClientEvent;
  progressionEvent: ProgressionEvent;
};

interface NotificationContext {
  notifications: NotificationItem[];
  ongoingNotifications: OngoingNotificationItem[];
  dispatch: React.Dispatch<NotificationAction>;
  ongoingDispatch: React.Dispatch<OngoingNotificationAction>;
}

export const NotificationContext = createContext<NotificationContext>({
  notifications: [],
  ongoingNotifications: [],
  dispatch: () => {
    console.error('dispatch not implemented');
  },
  ongoingDispatch: () => {
    console.error('ongoingDispatch not implemented');
  },
});

export const useNotificationReducer = () => {
  const [notifications, dispatch] = useReducer(
    (state: NotificationItem[], action: NotificationAction) => {
      const { message, status, event } = action;
      const key = event.snowflake_str;
      const timestamp = getSnowflakeTimestamp(event.snowflake_str);
      if (state.some((item) => item.key === key)) {
        console.warn('Notification with duplicate key received');
        return state;
      }
      return [
        ...state,
        { message, status, timestamp, key } as NotificationItem,
      ];
    },
    []
  );

  return { notifications, dispatch };
};

export const useOngoingNotificationReducer = () => {
  const [ongoingNotifications, ongoingDispatch] = useReducer(
    (state: OngoingNotificationItem[], action: OngoingNotificationAction) => {
      const { snowflake_str: snowflake } = action.event;
      const timestamp = getSnowflakeTimestamp(snowflake);
      const event_inner = action.progressionEvent.progression_event_inner;
      const event_id = action.progressionEvent.event_id;

      match(event_inner, {
        ProgressionStart: ({ progression_name, total }) => {
          state.push({
            state: 'ongoing',
            progress: 0,
            total,
            title: progression_name,
            message: null,
            timestamp,
            event_id,
            key: snowflake,
          });
        },
        ProgressionUpdate: ({ progress, progress_message }) => {
          state.map((item) => {
            if (item.event_id === event_id) {
              item.progress += progress;
              item.key = snowflake;
              if (progress_message) item.message = progress_message;
            }
          });
        },
        ProgressionEnd: ({ success, message }) => {
          state.map((item) => {
            if (item.event_id === event_id) {
              item.state = success ? 'done' : 'error';
              item.progress = item.total ?? 0;
              item.key = snowflake;
              if (message) item.message = message;
            }
          });
        },
      });
      return [...state];
    },
    []
  );
  return { ongoingNotifications, ongoingDispatch };
};
