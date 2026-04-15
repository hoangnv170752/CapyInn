import { useEffect, useState } from "react";
import type { OnboardingDraft, OnboardingRoomTypeDraft } from "./types";

const STORAGE_KEY = "mhm-onboarding-draft";

export function createRoomTypeDraft(): OnboardingRoomTypeDraft {
  return {
    tempId: `room-type-${Math.random().toString(36).slice(2, 10)}`,
    name: "",
    basePrice: 0,
    maxGuests: 2,
    extraPersonFee: 0,
    defaultHasBalcony: false,
    bedNote: "",
  };
}

const DEFAULT_DRAFT: OnboardingDraft = {
  locale: "vi",
  hotel: {
    name: "",
    address: "",
    phone: "",
    defaultCheckinTime: "14:00",
    defaultCheckoutTime: "12:00",
  },
  roomTypes: [createRoomTypeDraft()],
  generatedRooms: [],
  roomPlan: {
    floors: 1,
    roomsPerFloor: 1,
    namingScheme: "floor_letter",
    columnAssignments: [""],
  },
  appLock: {
    enabled: true,
    adminName: "",
    pin: "",
    confirmPin: "",
  },
};

export function useOnboardingDraft() {
  const [draft, setDraft] = useState<OnboardingDraft>(() => {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_DRAFT;

    try {
      return JSON.parse(raw) as OnboardingDraft;
    } catch {
      return DEFAULT_DRAFT;
    }
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(draft));
  }, [draft]);

  return {
    draft,
    setDraft,
    clearDraft: () => localStorage.removeItem(STORAGE_KEY),
  };
}
