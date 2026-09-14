import {
  HugeiconsIcon,
  type HugeiconsProps,
  type IconSvgElement,
} from "@hugeicons/react";
import React, { createElement, forwardRef } from "react";

import {
  ActivityIcon,
  AlertCircleIcon,
  AlertTriangle as HugeAlertTriangle,
  ArrowLeft01Icon,
  ArrowRight01Icon,
  AudioWave01Icon,
  BookOpen01Icon,
  BookTextIcon,
  Bug01Icon,
  Cancel01Icon,
  CheckmarkCircle02Icon,
  ChevronDownIcon as HugeChevronDownIcon,
  ChevronLeftIcon as HugeChevronLeftIcon,
  ChevronRightIcon as HugeChevronRightIcon,
  ChevronUpIcon as HugeChevronUpIcon,
  CircleIcon as HugeCircleIcon,
  ClipboardIcon,
  Clock01Icon,
  ComputerIcon,
  Copy01Icon,
  CpuIcon,
  DashboardSpeed01Icon,
  DashboardSquare01Icon,
  Delete01Icon,
  Download01Icon,
  Drag01Icon,
  FileAudioIcon,
  Fire02Icon,
  FlashIcon,
  Globe02Icon,
  HardDriveIcon,
  HeadphonesIcon,
  HelpCircleIcon,
  HistoryIcon as HugeHistoryIcon,
  InformationCircleIcon,
  Key01Icon,
  KeyboardIcon,
  LinkSquare01Icon,
  Loading03Icon,
  MagicWand01Icon,
  Maximize01Icon,
  Message01Icon,
  MessageSquareHeartIcon,
  Mic01Icon,
  MicOff01Icon,
  MinusSignIcon,
  MoreHorizontalCircle01Icon,
  OctagonXIcon as HugeOctagonXIcon,
  PowerIcon,
  Refresh01Icon,
  RefreshCwIcon,
  RotateLeft01Icon,
  Search01Icon,
  Settings01Icon,
  Share01Icon,
  ShieldAlertIcon,
  ShieldCheckIcon,
  ShieldXIcon,
  SidebarLeftIcon,
  SlidersHorizontalIcon,
  SparklesIcon,
  StarIcon,
  TextIcon,
  Tick02Icon,
  Timer01Icon,
  TriangleAlertIcon as HugeTriangleAlertIcon,
  Upload01Icon,
  VolumeHighIcon,
  Wrench01Icon,
} from "@hugeicons/core-free-icons";

export type IconProps = React.SVGProps<SVGSVGElement> & {
  size?: string | number;
  primaryColor?: string;
  secondaryColor?: string;
  disableSecondaryOpacity?: boolean;
};

export type IconComponent = React.ForwardRefExoticComponent<
  IconProps & React.RefAttributes<SVGSVGElement>
>;

export function createHugeIcon(
  icon: IconSvgElement,
  displayName?: string
): IconComponent {
  const Component = forwardRef<SVGSVGElement, IconProps>((props, ref) => {
    const { strokeWidth, ...rest } = props;
    const numStrokeWidth =
      typeof strokeWidth === "string" ? parseFloat(strokeWidth) : strokeWidth;

    return createElement(HugeiconsIcon, {
      ref,
      icon,
      strokeWidth:
        typeof numStrokeWidth === "number" && !isNaN(numStrokeWidth)
          ? numStrokeWidth
          : 1.5,
      ...(rest as any),
    });
  });
  if (displayName) {
    Component.displayName = displayName;
  }
  return Component;
}

// Export raw Hugeicons component and props
export { HugeiconsIcon };
export type { HugeiconsProps, IconSvgElement };

// Icon components mapped for application and shadcn UI usage
export const Activity = createHugeIcon(ActivityIcon, "Activity");
export const AlertCircle = createHugeIcon(AlertCircleIcon, "AlertCircle");
export const AlertTriangle = createHugeIcon(HugeAlertTriangle, "AlertTriangle");
export const ArrowLeft = createHugeIcon(ArrowLeft01Icon, "ArrowLeft");
export const ArrowRight = createHugeIcon(ArrowRight01Icon, "ArrowRight");
export const AudioWave = createHugeIcon(AudioWave01Icon, "AudioWave");
export const BookOpen = createHugeIcon(BookOpen01Icon, "BookOpen");
export const BookText = createHugeIcon(BookTextIcon, "BookText");
export const Bug = createHugeIcon(Bug01Icon, "Bug");
export const Check = createHugeIcon(Tick02Icon, "Check");
export const CheckIcon = createHugeIcon(Tick02Icon, "CheckIcon");
export const ChevronDownIcon = createHugeIcon(HugeChevronDownIcon, "ChevronDownIcon");
export const ChevronLeftIcon = createHugeIcon(HugeChevronLeftIcon, "ChevronLeftIcon");
export const ChevronRight = createHugeIcon(HugeChevronRightIcon, "ChevronRight");
export const ChevronRightIcon = createHugeIcon(HugeChevronRightIcon, "ChevronRightIcon");
export const ChevronUpIcon = createHugeIcon(HugeChevronUpIcon, "ChevronUpIcon");
export const Circle = createHugeIcon(HugeCircleIcon, "Circle");
export const CircleCheckIcon = createHugeIcon(CheckmarkCircle02Icon, "CircleCheckIcon");
export const CircleIcon = createHugeIcon(HugeCircleIcon, "CircleIcon");
export const Clipboard = createHugeIcon(ClipboardIcon, "Clipboard");
export const Clock = createHugeIcon(Clock01Icon, "Clock");
export const Copy = createHugeIcon(Copy01Icon, "Copy");
export const Cpu = createHugeIcon(CpuIcon, "Cpu");
export const Download = createHugeIcon(Download01Icon, "Download");
export const ExternalLink = createHugeIcon(LinkSquare01Icon, "ExternalLink");
export const FileAudio = createHugeIcon(FileAudioIcon, "FileAudio");
export const Flame = createHugeIcon(Fire02Icon, "Flame");
export const Gauge = createHugeIcon(DashboardSpeed01Icon, "Gauge");
export const Globe = createHugeIcon(Globe02Icon, "Globe");
export const GripVerticalIcon = createHugeIcon(Drag01Icon, "GripVerticalIcon");
export const HardDrive = createHugeIcon(HardDriveIcon, "HardDrive");
export const Headphones = createHugeIcon(HeadphonesIcon, "Headphones");
export const HelpCircle = createHugeIcon(HelpCircleIcon, "HelpCircle");
export const History = createHugeIcon(HugeHistoryIcon, "History");
export const HistoryIcon = createHugeIcon(HugeHistoryIcon, "HistoryIcon");
export const Info = createHugeIcon(InformationCircleIcon, "Info");
export const InfoIcon = createHugeIcon(InformationCircleIcon, "InfoIcon");
export const Key = createHugeIcon(Key01Icon, "Key");
export const Keyboard = createHugeIcon(KeyboardIcon, "Keyboard");
export const LayoutDashboard = createHugeIcon(DashboardSquare01Icon, "LayoutDashboard");
export const Loader2 = createHugeIcon(Loading03Icon, "Loader2");
export const Loader2Icon = createHugeIcon(Loading03Icon, "Loader2Icon");
export const Maximize2 = createHugeIcon(Maximize01Icon, "Maximize2");
export const MessageSquare = createHugeIcon(Message01Icon, "MessageSquare");
export const MessageSquareHeart = createHugeIcon(MessageSquareHeartIcon, "MessageSquareHeart");
export const Mic = createHugeIcon(Mic01Icon, "Mic");
export const MicOff = createHugeIcon(MicOff01Icon, "MicOff");
export const MinusIcon = createHugeIcon(MinusSignIcon, "MinusIcon");
export const Monitor = createHugeIcon(ComputerIcon, "Monitor");
export const MoreHorizontal = createHugeIcon(MoreHorizontalCircle01Icon, "MoreHorizontal");
export const MoreHorizontalIcon = createHugeIcon(MoreHorizontalCircle01Icon, "MoreHorizontalIcon");
export const OctagonXIcon = createHugeIcon(HugeOctagonXIcon, "OctagonXIcon");
export const PanelLeftIcon = createHugeIcon(SidebarLeftIcon, "PanelLeftIcon");
export const Power = createHugeIcon(PowerIcon, "Power");
export const RefreshCcw = createHugeIcon(Refresh01Icon, "RefreshCcw");
export const RefreshCw = createHugeIcon(RefreshCwIcon, "RefreshCw");
export const RotateCcw = createHugeIcon(RotateLeft01Icon, "RotateCcw");
export const Search = createHugeIcon(Search01Icon, "Search");
export const SearchIcon = createHugeIcon(Search01Icon, "SearchIcon");
export const Settings = createHugeIcon(Settings01Icon, "Settings");
export const Share2 = createHugeIcon(Share01Icon, "Share2");
export const ShieldAlert = createHugeIcon(ShieldAlertIcon, "ShieldAlert");
export const ShieldCheck = createHugeIcon(ShieldCheckIcon, "ShieldCheck");
export const ShieldX = createHugeIcon(ShieldXIcon, "ShieldX");
export const Sliders = createHugeIcon(SlidersHorizontalIcon, "Sliders");
export const Sparkles = createHugeIcon(SparklesIcon, "Sparkles");
export const Star = createHugeIcon(StarIcon, "Star");
export const Timer = createHugeIcon(Timer01Icon, "Timer");
export const Trash2 = createHugeIcon(Delete01Icon, "Trash2");
export const TriangleAlertIcon = createHugeIcon(HugeTriangleAlertIcon, "TriangleAlertIcon");
export const Type = createHugeIcon(TextIcon, "Type");
export const Upload = createHugeIcon(Upload01Icon, "Upload");
export const Volume2 = createHugeIcon(VolumeHighIcon, "Volume2");
export const Wand2 = createHugeIcon(MagicWand01Icon, "Wand2");
export const Waves = createHugeIcon(AudioWave01Icon, "Waves");
export const Wrench = createHugeIcon(Wrench01Icon, "Wrench");
export const X = createHugeIcon(Cancel01Icon, "X");
export const XIcon = createHugeIcon(Cancel01Icon, "XIcon");
export const Zap = createHugeIcon(FlashIcon, "Zap");
