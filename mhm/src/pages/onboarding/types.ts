export interface OnboardingRoomTypeDraft {
  tempId: string;
  name: string;
  basePrice: number;
  maxGuests: number;
  extraPersonFee: number;
  defaultHasBalcony: boolean;
  bedNote?: string;
}

export interface OnboardingGeneratedRoom {
  id: string;
  name: string;
  floor: number;
  roomTypeName: string;
  hasBalcony: boolean;
  basePrice: number;
  maxGuests: number;
  extraPersonFee: number;
}

export interface OnboardingDraft {
  locale: "vi" | "en";
  hotel: {
    name: string;
    address: string;
    phone: string;
    rating?: string;
    defaultCheckinTime: string;
    defaultCheckoutTime: string;
  };
  roomTypes: OnboardingRoomTypeDraft[];
  generatedRooms: OnboardingGeneratedRoom[];
  roomPlan: {
    floors: number;
    roomsPerFloor: number;
    namingScheme: "floor_letter" | "floor_number" | "custom";
    columnAssignments: string[];
  };
  appLock: {
    enabled: boolean;
    adminName: string;
    pin: string;
    confirmPin: string;
  };
}
