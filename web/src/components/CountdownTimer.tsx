/**
 * CountdownTimer - Displays countdown to/from a deadline.
 * 
 * Expected Behavior:
 *   Shows days/hours/minutes/seconds countdown.
 *   Updates every second.
 *   Displays different states: upcoming, active, ended.
 *   Emits event when countdown reaches zero.
 * 
 * Args:
 *   targetDate: Target date/time for countdown.
 *   onComplete: Optional callback when countdown ends.
 *   label: Optional label text.
 * 
 * Returns:
 *   Countdown display component.
 */
'use client';

import { useState, useEffect, useCallback } from 'react';
import { Clock, Play, CheckCircle, AlertCircle } from 'lucide-react';

interface CountdownTimerProps {
  targetDate: Date | string;
  label?: string;
  onComplete?: () => void;
  showLabel?: boolean;
  size?: 'sm' | 'md' | 'lg';
}

interface TimeRemaining {
  days: number;
  hours: number;
  minutes: number;
  seconds: number;
  total: number;
  isExpired: boolean;
}

export function CountdownTimer({
  targetDate,
  label,
  onComplete,
  showLabel = true,
  size = 'md',
}: CountdownTimerProps) {
  const [timeRemaining, setTimeRemaining] = useState<TimeRemaining>({
    days: 0,
    hours: 0,
    minutes: 0,
    seconds: 0,
    total: 0,
    isExpired: false,
  });

  const calculateTimeRemaining = useCallback((target: Date): TimeRemaining => {
    const total = target.getTime() - new Date().getTime();
    const isExpired = total <= 0;
    const absTotal = Math.abs(total);

    return {
      days: Math.floor(absTotal / (1000 * 60 * 60 * 24)),
      hours: Math.floor((absTotal / (1000 * 60 * 60)) % 24),
      minutes: Math.floor((absTotal / 1000 / 60) % 60),
      seconds: Math.floor((absTotal / 1000) % 60),
      total,
      isExpired,
    };
  }, []);

  useEffect(() => {
    const target = new Date(targetDate);
    
    // Initial calculation
    setTimeRemaining(calculateTimeRemaining(target));

    // Update every second
    const timer = setInterval(() => {
      const remaining = calculateTimeRemaining(target);
      setTimeRemaining(remaining);

      // Trigger onComplete when countdown reaches zero
      if (remaining.total <= 0 && !remaining.isExpired && onComplete) {
        onComplete();
      }
    }, 1000);

    return () => clearInterval(timer);
  }, [targetDate, calculateTimeRemaining, onComplete]);

  const formatNumber = (num: number): string => {
    return num.toString().padStart(2, '0');
  };

  const getStatusInfo = () => {
    if (timeRemaining.isExpired) {
      return {
        text: 'Ended',
        color: 'text-red-600 dark:text-red-400',
        bgColor: 'bg-red-50 dark:bg-red-900/20',
        borderColor: 'border-red-200 dark:border-red-800',
        icon: <CheckCircle className="w-5 h-5" />,
      };
    }
    if (timeRemaining.total < 0) {
      return {
        text: 'Starting in...',
        color: 'text-blue-600 dark:text-blue-400',
        bgColor: 'bg-blue-50 dark:bg-blue-900/20',
        borderColor: 'border-blue-200 dark:border-blue-800',
        icon: <Clock className="w-5 h-5" />,
      };
    }
    return {
      text: label || 'Time Remaining',
      color: 'text-green-600 dark:text-green-400',
      bgColor: 'bg-green-50 dark:bg-green-900/20',
      borderColor: 'border-green-200 dark:border-green-800',
      icon: <Play className="w-5 h-5" />,
    };
  };

  const status = getStatusInfo();

  const sizeClasses = {
    sm: {
      container: 'p-2',
      number: 'text-lg font-bold',
      label: 'text-xs',
      icon: 'w-4 h-4',
    },
    md: {
      container: 'p-4',
      number: 'text-2xl font-bold',
      label: 'text-sm',
      icon: 'w-5 h-5',
    },
    lg: {
      container: 'p-6',
      number: 'text-4xl font-bold',
      label: 'text-base',
      icon: 'w-6 h-6',
    },
  };

  return (
    <div className={`inline-block ${status.bgColor} ${status.borderColor} border rounded-lg ${sizeClasses[size].container}`}>
      {showLabel && (
        <div className={`flex items-center gap-2 mb-2 ${status.color}`}>
          {status.icon}
          <span className="text-sm font-medium">{status.text}</span>
        </div>
      )}
      
      <div className="flex items-center gap-2 font-mono">
        {timeRemaining.days > 0 && (
          <div className="text-center">
            <div className={`${sizeClasses[size].number} ${status.color}`}>
              {formatNumber(timeRemaining.days)}
            </div>
            <div className={`${sizeClasses[size].label} text-gray-500 dark:text-gray-400`}>
              days
            </div>
          </div>
        )}
        
        <div className="text-center">
          <div className={`${sizeClasses[size].number} ${status.color}`}>
            {formatNumber(timeRemaining.hours)}
          </div>
          <div className={`${sizeClasses[size].label} text-gray-500 dark:text-gray-400`}>
            hours
          </div>
        </div>
        
        <span className={`${status.color} text-2xl`}>:</span>
        
        <div className="text-center">
          <div className={`${sizeClasses[size].number} ${status.color}`}>
            {formatNumber(timeRemaining.minutes)}
          </div>
          <div className={`${sizeClasses[size].label} text-gray-500 dark:text-gray-400`}>
            min
          </div>
        </div>
        
        <span className={`${status.color} text-2xl`}>:</span>
        
        <div className="text-center">
          <div className={`${sizeClasses[size].number} ${status.color}`}>
            {formatNumber(timeRemaining.seconds)}
          </div>
          <div className={`${sizeClasses[size].label} text-gray-500 dark:text-gray-400`}>
            sec
          </div>
        </div>
      </div>

      {timeRemaining.total < 0 && timeRemaining.total > -60000 && (
        <div className="mt-2 text-xs text-blue-600 dark:text-blue-400 animate-pulse">
          Starting soon...
        </div>
      )}

      {timeRemaining.total < -60000 && (
        <div className="mt-2 text-xs text-red-600 dark:text-red-400">
          Ended {Math.abs(Math.floor(timeRemaining.total / 60000))}m ago
        </div>
      )}
    </div>
  );
}

/**
 * PhaseCountdown - Shows countdown for current hackathon phase.
 * 
 * Expected Behavior:
 *   Fetches current phase from API.
 *   Displays appropriate countdown based on phase state.
 *   Auto-refreshes phase data.
 */
interface PhaseCountdownProps {
  hackathonId: string;
}

export function PhaseCountdown({ hackathonId }: PhaseCountdownProps) {
  const [currentPhase, setCurrentPhase] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchPhase = async () => {
      try {
        const res = await fetch(`/api/core/phases/hackathon/${hackathonId}/current`);
        const data = await res.json();
        setCurrentPhase(data.current_phase);
      } catch (error) {
        console.error('Failed to fetch current phase:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchPhase();
    const interval = setInterval(fetchPhase, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, [hackathonId]);

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-gray-500">
        <Clock className="w-5 h-5 animate-spin" />
        <span>Loading phase...</span>
      </div>
    );
  }

  if (!currentPhase) {
    return (
      <div className="text-gray-500">
        No active phase
      </div>
    );
  }

  const targetDate = currentPhase.is_active 
    ? currentPhase.closes_at 
    : currentPhase.opens_at;

  const label = currentPhase.is_active
    ? `${currentPhase.name} ends in`
    : `${currentPhase.name} starts in`;

  return (
    <div className="space-y-2">
      <div className="text-sm text-gray-600 dark:text-gray-400">
        Current Phase: <span className="font-semibold">{currentPhase.name}</span>
        {currentPhase.type && (
          <span className="ml-2 px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-800 rounded">
            {currentPhase.type}
          </span>
        )}
      </div>
      <CountdownTimer
        targetDate={targetDate}
        label={label}
        size="lg"
        showLabel={true}
      />
    </div>
  );
}
