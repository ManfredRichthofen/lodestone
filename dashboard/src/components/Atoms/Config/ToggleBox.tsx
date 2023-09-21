import { useEffect, useState } from 'react';
import BeatLoader from 'react-spinners/BeatLoader';
import { catchAsyncToString } from 'utils/util';
import { Switch } from '@headlessui/react';
import { Toggle } from '../Toggle';

/**
 * A self controlled toggle component meant to represent a single value of a config
 *
 * It is NOT meant to be used as a form input
 *
 * See ToggleField for that
 */
export default function ToggleBox({
  label,
  value: initialValue,
  className,
  onChange: onChangeProp,
  error: errorProp,
  disabled = false,
  canRead = true,
  isLoading: isLoadingProp = false,
  description,
  descriptionFunc,
  optimistic = true, // if true, the toggle will change immediately and go into loading state, and will change back if onChange throws an error
}: {
  label: string;
  value: boolean;
  className?: string;
  error?: string;
  disabled?: boolean;
  canRead?: boolean;
  isLoading?: boolean;
  onChange: (arg: boolean) => Promise<void>;
  description?: React.ReactNode;
  descriptionFunc?: (arg: boolean) => React.ReactNode;
  optimistic?: boolean;
}) {
  const [value, setValue] = useState(initialValue);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  // set value to initialValue when initialValue changes
  useEffect(() => {
    setValue(initialValue);
  }, [initialValue]);

  const onChange = async (newValue: boolean) => {
    if (optimistic) setValue(newValue);
    setIsLoading(true);
    const submitError = await catchAsyncToString(onChangeProp(newValue));
    setError(submitError);
    setIsLoading(false);
    if (submitError.length > 0) {
      setValue(initialValue);
    }
  };

  const errorText = errorProp || error;
  disabled = disabled || !canRead || isLoadingProp;
  description = canRead
    ? descriptionFunc?.(initialValue || value) ?? description
    : 'No permission';

  const status =
    isLoading || isLoadingProp ? (
      <BeatLoader
        key="loading"
        size="0.25rem"
        cssOverride={{
          width: '2rem',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          margin: `0 -0.5rem`,
        }}
        color="#6b7280"
      />
    ) : (
      <p className="text-small font-medium italic text-white/50">
        {disabled ? '' : value ? 'Enabled' : 'Disabled'}
      </p>
    );

  return (
    <div
      className={`flex flex-row items-center justify-between ${className} group relative gap-4 bg-gray-800 px-4 py-3 text-h3`}
    >
      <div className={`flex min-w-0 grow flex-col`}>
        <label className="text-medium font-medium tracking-medium text-gray-300">
          {label}
        </label>
        {errorText ? (
          <div className="text-small font-medium tracking-medium text-red">
            {errorText || 'Unknown error'}
          </div>
        ) : (
          <div className="overflow-hidden text-ellipsis text-medium font-medium tracking-medium text-white/50">
            {description}
          </div>
        )}
      </div>
      <div className="relative flex w-5/12 shrink-0 flex-row items-center justify-end gap-4">
        {status}
        <Toggle
          value={value}
          onChange={onChange}
          disabled={disabled || isLoading}
        />
      </div>
    </div>
  );
}
