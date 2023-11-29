import React, { useState } from "react";

export const ControlPanel = ({
	buttonInputKeys,
	handleInputChange,
	initialPayloads,
	lastClickedButton,
	payloads,
	setLastClickedButton,
	setPayloads,
	setShowChat,
	setShowLogs,
}) => {
	const [hoveredButton, setHoveredButton] = useState(null);
	const settingsOptions = {
		write_inside_repository: ["true", "false"],
		llm: ["gpt-3.5-turbo-1106", "gpt-4-1106-preview"],
		embedding_model: ["text-embedding-ada-002"],
	};
	const [showSettings, setShowSettings] = useState(false);

	const handleButtonClick = async (index) => {
		setLastClickedButton(index);
		setShowLogs(false);

		const requiredFields = buttonInputKeys[index];
		const missingFields = requiredFields.filter((field) => !payloads[field]);

		if (missingFields.length > 0 || index !== 3) {
			setShowChat(false);
		}
	};

	const handleSelectOption = (selectedOptions, key) => {
		setPayloads((prevPayloads) => ({
			...prevPayloads,
			[key]: selectedOptions,
		}));
	};

	const handleSettingsButtonClick = () => {
		setShowSettings(!showSettings);
	};

	const shouldHighlight = (payloadKey) => {
		if (hoveredButton === null) return false;
		const hoveredKeys = buttonInputKeys[hoveredButton];
		return hoveredKeys.includes(payloadKey);
	};

	return (
		<div>
			{showSettings && (
				<div
					className={`gap-4 grid md:grid-cols-2 mt-4 px-20 ${
						showSettings ? "" : "hidden"
					}`}
				>
					{Object.keys(initialPayloads).map((key, index) => (
						<div key={index} className="flex items-center justify-between">
							<label className="text-docBlue text-xs">
								{
									[
										"Repository path:",
										"Write to JSON:",
										"Write in repository:",
										"OpenAI api key:",
										"LLM:",
										"Embedding model:",
										"Qdrant url:",
										"Qdrant collection name:",
									][index]
								}
							</label>
							{settingsOptions[key] ? (
								<select
									className={`bg-docGrey4 border border-2 border-docGrey4 cursor-pointer focus:outline-none focus:border-docViolet mt-2 mx-2 p-2 rounded text-docWhite text-xs w-40 ${
										shouldHighlight(key) ? "highlight" : ""
									}`}
									value={payloads[key]}
									onChange={(e) =>
										handleSelectOption(
											Array.from(
												e.target.selectedOptions,
												(option) => option.value
											),
											key
										)
									}
								>
									<option value="" disabled>
										Select an option
									</option>
									{settingsOptions[key].map((option) => (
										<option key={option} value={option}>
											{option}
										</option>
									))}
								</select>
							) : (
								<input
									type="text"
									className={`bg-docGrey4 border border-2 border-docGrey4 focus:outline-none focus:border-docViolet mt-2 mx-2 p-2 placeholder-docGrey1 rounded text-docWhite text-xs w-40 ${
										shouldHighlight(key) ? "highlight" : ""
									}`}
									placeholder={
										[
											"~/rusty-docs/src",
											"",
											"",
											"OpenAI api key",
											"",
											"",
											"http://qdrant-db.my-qdrant:6334",
											"",
										][index]
									}
									value={payloads[key]}
									onChange={(e) => handleInputChange(key, e)}
								/>
							)}
						</div>
					))}
				</div>
			)}

			<div className="flex justify-center mt-10 text-docYellow text-lg">
				<div className="flex flex-row">
					{["Parse", "Document", "Embed", "Ask"].map((buttonLabel, index) => (
						<div key={index} className="hover:font-bold mb-8">
							<button
								className={`h-10 mx-5 md:w-24 ${
									lastClickedButton === index ? "font-bold text-docGreen" : ""
								}`}
								onClick={() => handleButtonClick(index)}
								onMouseEnter={() => setHoveredButton(index)}
								onMouseLeave={() => setHoveredButton(null)}
							>
								{buttonLabel}
							</button>
						</div>
					))}
					<button
						className={`h-10 mx-5 ${showSettings ? "text-docGreen" : ""}`}
						onClick={handleSettingsButtonClick}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
							strokeWidth="1.5"
							stroke="currentColor"
							className="w-6 h-6"
						>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z"
							/>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
							/>
						</svg>
					</button>
				</div>
			</div>
			<style jsx>{`
				.highlight {
					border-color: #b78cf2;
				}
			`}</style>
		</div>
	);
};
