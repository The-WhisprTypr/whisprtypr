import { useAppStore } from "@/store";
import { CompleteStep } from "./complete-step";
import { HotkeyStep } from "./hotkey-step";
import { LicenseStep } from "./license-step";
import { MicrophoneStep } from "./microphone-step";
import { ModelSelectStep } from "./model-select-step";
import { WelcomeStep } from "./welcome-step";

export function SetupWizard() {
  const { currentSetupStep, nextSetupStep, prevSetupStep, setSetupComplete } =
    useAppStore();

  const handleFinish = () => {
    setSetupComplete(true);
  };

  // Render the appropriate step based on currentSetupStep
  // Steps: 0=Welcome, 1=License, 2=Microphone, 3=Model, 4=Hotkey, 5=Complete
  switch (currentSetupStep) {
    case 0:
      return <WelcomeStep onNext={nextSetupStep} />;
    case 1:
      return <LicenseStep onNext={nextSetupStep} onBack={prevSetupStep} />;
    case 2:
      return <MicrophoneStep onNext={nextSetupStep} onBack={prevSetupStep} />;
    case 3:
      return <ModelSelectStep onNext={nextSetupStep} onBack={prevSetupStep} />;
    case 4:
      return <HotkeyStep onNext={nextSetupStep} onBack={prevSetupStep} />;
    case 5:
      return <CompleteStep onFinish={handleFinish} />;
    default:
      return <WelcomeStep onNext={nextSetupStep} />;
  }
}
